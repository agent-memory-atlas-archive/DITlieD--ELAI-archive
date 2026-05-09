// ============================================================
// activity.svelte.ts — Activity stream state
//
// Phase B wiring: drop the ACTIVITY_ENTRIES mock seed. Initial
// list is empty. Entries are appended in two ways:
//   1) by the Tauri event subscriber installed in Workspace
//      onMount (planned for Phase D — emits `activity-entry` from
//      orchestrator on every reasoning_trace / edit / validate / etc.)
//   2) by frontend code that wants to surface a transient note
//      (intervention response, error card, etc.) via append().
// ============================================================

import type { ActivityEntry } from '../api/types';
import { logStub } from '../api/stub';

const MAX_ENTRIES = 200;

function createActivity() {
  let entries = $state<ActivityEntry[]>([]);
  let autoScroll = $state<boolean>(true);
  let scrollToStepId = $state<string | null>(null);

  return {
    get entries() { return entries; },
    get autoScroll() { return autoScroll; },
    get scrollToStepId() { return scrollToStepId; },

    /** Phase D will attach a Tauri event listener here. Currently a no-op. */
    bootstrap() {
      logStub('activity.bootstrap');
    },

    teardown() {},

    append(entry: ActivityEntry) {
      const next = [...entries, entry];
      entries = next.length > MAX_ENTRIES ? next.slice(next.length - MAX_ENTRIES) : next;
    },

    clear() {
      entries = [];
    },

    setAutoScroll(v: boolean) {
      autoScroll = v;
    },

    jumpToLatest() {
      autoScroll = true;
    },

    scrollTo(stepId: string) {
      scrollToStepId = stepId;
    },

    clearScrollTo() {
      scrollToStepId = null;
    },
  };
}

export const activity = createActivity();
