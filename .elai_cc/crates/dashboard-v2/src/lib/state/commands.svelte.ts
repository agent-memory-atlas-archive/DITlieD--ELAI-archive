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
  query_knowledge_engine,
  continue_task,
  get_media_validations,
  get_media_validation_by_step,
  submit_media_validation_for_review,
  type ResearchResult,
} from '../api/stub';
import { classifyIntent } from '../utils/intent';
import { parseSlashCommand, KNOWN_SLASH_COMMANDS } from '../utils/slash-parse';
import { workspace } from './workspace.svelte';
import { mediaValidation } from './media_validation.svelte';

// B-24 Phase 4I Group C8 — re-export the 4 media commands so consumers
// that import from 'state/commands' have a single entry point. The
// reactive store lives in `media_validation.svelte.ts`; these wrappers
// expose the imperative commands for ad-hoc calls (e.g. from intent
// dispatch or from a future slash command).
export const mediaCommands = {
  list: get_media_validations,
  byStep: get_media_validation_by_step,
  submitReview: submit_media_validation_for_review,
  // Reactive store accessor — components can `import { mediaValidation }`
  // directly from 'state/media_validation.svelte' but re-exporting here
  // keeps the dispatch surface symmetric.
  store: mediaValidation,
};

function makeId() { return `cc-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`; }

const SLASH_COMMANDS = KNOWN_SLASH_COMMANDS;

// Slash commands whose backend dispatch is currently a TODO. These still
// route through the slash path so the user sees a clear error card instead
// of silently streaming the literal text into chat.
//
// /ask is in this set: in CLI it's an alias for /research, but the dashboard
// dispatcher wires /research (which calls run_research) and not /ask. The
// auto-detect Oracle gate therefore routes accept-chip through /research,
// not /ask, to hit the working backend.
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
  // Auto-detect Oracle gate. When the classifier returns RESEARCH on a
  // non-slash submit, we hold the text in this slot and surface a confirm
  // chip in CommandWindow instead of firing the request. Three resolution
  // paths: acceptResearchChip / rejectResearchChip / editResearchChip.
  // Null = no pending chip.
  let pendingResearchChip = $state<{ text: string } | null>(null);
  // Signal CommandWindow to refocus the textarea after Edit. Increments on
  // each editResearchChip call; the window's $effect on this counter calls
  // textareaEl.focus(). Avoids passing DOM refs through the store.
  let _refocusInputTick = $state(0);

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

  async function _runStreamingPath(text: string, intent: IntentType) {
    logStub('commands.submit', { text, intent });

    const requestId = makeId();
    activeStreamId = requestId;

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
          cards = cards.filter(
            (c) => !(c.kind === 'response' && (c.payload as ResponseCardPayload).stream_id === requestId),
          );
          appendCard('error', {
            kind: 'error',
            message: errMsg || (offline ? 'upstream offline' : 'unknown error'),
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

  async function submit(text: string) {
    if (!text.trim()) return;

    const intent = classifyIntent(text);
    const slash = parseSlashCommand(text);

    // Auto-detect Oracle gate. Fires only when ALL hold:
    //   - non-slash input (slash bypass is the explicit power-user path)
    //   - classifier inferred RESEARCH from the regex layer
    //   - no stream is in flight (popping a chip mid-conversation = jarring)
    //   - no chip already pending (one-at-a-time)
    // When fired: hold the text on pendingResearchChip and DEFER user-card
    // append + dispatch until the chip resolves. Editing the chip rolls back
    // cleanly because nothing was added to the stream yet.
    if (
      slash === null
      && intent === 'RESEARCH'
      && !isStreaming
      && pendingResearchChip === null
    ) {
      pendingResearchChip = { text };
      inputValue = '';
      slashOpen = false;
      return;
    }

    // Standard path: user-card first, then slash dispatch or local stream.
    appendCard('user', { kind: 'user', text, intent });
    inputValue = '';
    slashOpen = false;

    // EXPLORE_CODE wire (B-explore-code): route codebase questions to the
    // local BM25 index for the active project. Short-circuit before the
    // streaming LLM path so "what calls Token::validate?" returns ranked
    // file hits instead of model hallucination. /explore slash command
    // takes the same path. Vector/cross-encoder degraded — see knowledge
    // engine audit (DETAILED_SUMMARY_01 §2 KNOWLEDGE).
    if (intent === 'EXPLORE_CODE' || (slash !== null && slash.command === '/explore')) {
      const queryText = slash !== null && slash.arg ? slash.arg : text;
      logStub('commands.submit.explore_code', { query: queryText });
      try {
        const answer = await query_knowledge_engine(queryText);
        appendCard('code-answer', {
          kind: 'code-answer',
          query: answer.query,
          hits: answer.hits,
          status: answer.status,
          message: answer.message,
        });
      } catch (e) {
        appendCard('error', {
          kind: 'error',
          message: `knowledge query failed: ${String(e)}`,
          source: 'sidecar',
          step_id: undefined,
          recoverable: true,
        });
      }
      return;
    }

    // B-24 Phase 4I Group C8: route MEDIA_VALIDATE intent to the
    // MediaValidationPanel inside MonitorWindow. We refresh the panel's
    // store so the latest entries are visible, then open the monitor tab.
    if (intent === 'MEDIA_VALIDATE') {
      logStub('commands.submit.media_validate', { text });
      try {
        await mediaValidation.refresh();
      } catch (e) {
        // Non-fatal — the panel renders an error state on its own.
        logStub('commands.submit.media_validate.refresh_error', { error: String(e) });
      }
      workspace.openTab('primary', 'monitor');
      appendCard('status', {
        kind: 'status',
        message:
          'opened the media validation panel in Monitor. ' +
          'Pending HumanReview rows are actionable; Tier1/Tier2 results stream live from SSB.',
        fsm_state: undefined,
        step_id: undefined,
      });
      return;
    }

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

    await _runStreamingPath(text, intent);
  }

  async function acceptResearchChip() {
    const chip = pendingResearchChip;
    if (!chip) return;
    pendingResearchChip = null;
    appendCard('user', { kind: 'user', text: chip.text, intent: 'RESEARCH' });
    logStub('commands.submit.slash', { command: '/research', arg: chip.text, intent: 'RESEARCH' });
    await dispatchSlash('/research', chip.text, 'RESEARCH');
  }

  async function rejectResearchChip() {
    const chip = pendingResearchChip;
    if (!chip) return;
    pendingResearchChip = null;
    appendCard('user', { kind: 'user', text: chip.text, intent: 'RESEARCH' });
    await _runStreamingPath(chip.text, 'RESEARCH');
  }

  function editResearchChip() {
    const chip = pendingResearchChip;
    if (!chip) return;
    inputValue = chip.text;
    pendingResearchChip = null;
    _refocusInputTick = _refocusInputTick + 1;
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
    get pendingResearchChip() { return pendingResearchChip; },
    get refocusInputTick() { return _refocusInputTick; },

    submit,
    appendCard,
    cancelStream,
    setInput,
    openSlash,
    closeSlash,
    setFocused,
    acceptResearchChip,
    rejectResearchChip,
    editResearchChip,
  };
}

export const commands = createCommands();
