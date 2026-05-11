// ============================================================
// media_validation.svelte.ts — Svelte 5 runes store for the
// B-24 Phase 4I Group C8 media validation panel.
// ============================================================
//
// Pattern mirrors fsm.svelte.ts / session.svelte.ts: a single
// createMediaValidation() factory returns a reactive store object
// with getters and action methods. MonitorWindow imports the
// singleton and binds reactively via `$derived`.
//
// Event flow:
//   1. mount() — calls get_media_validations() to seed entries,
//      then attaches subscribe_media_ssb_events listener.
//   2. On every media_ssb event: re-fetch get_media_validations
//      (cheap: one HashMap clone in Rust). Avoids per-event diff
//      logic; the cache is small.
//   3. unmount() — invokes the unlisten fn returned by step 2.
// ============================================================

import {
  get_media_validations,
  get_media_validation_by_step,
  submit_media_validation_for_review,
  subscribe_media_ssb_events,
  logStub,
} from '../api/stub';
import type {
  MediaValidationEntry,
  MediaSsbEvent,
  ReviewDecision,
} from '../api/types';

function createMediaValidation() {
  let entries = $state<MediaValidationEntry[]>([]);
  let loading = $state<boolean>(false);
  let loadError = $state<string | null>(null);
  let lastEvent = $state<MediaSsbEvent | null>(null);
  let _unlisten: (() => void) | null = null;
  let _mounted = false;

  async function refresh(): Promise<void> {
    loading = true;
    try {
      entries = await get_media_validations();
      loadError = null;
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  }

  async function fetchOne(stepId: string): Promise<MediaValidationEntry | null> {
    try {
      return await get_media_validation_by_step(stepId);
    } catch (e) {
      logStub('media_validation.fetchOne.error', { stepId, error: String(e) });
      return null;
    }
  }

  async function submitReview(
    stepId: string,
    decision: ReviewDecision,
  ): Promise<{ ok: boolean; error?: string }> {
    const result = await submit_media_validation_for_review(stepId, decision);
    if (result.ok) {
      // Refresh to pick up the patched verdict; cheaper than mutating a
      // single entry in place because the Rust side is the source of truth.
      await refresh();
    }
    return result;
  }

  async function mount(): Promise<void> {
    if (_mounted) return;
    _mounted = true;
    await refresh();
    try {
      _unlisten = await subscribe_media_ssb_events((event) => {
        lastEvent = event;
        // Re-fetch all entries on any media_* event. Lazy + correct.
        void refresh();
      });
    } catch (e) {
      loadError = `media_ssb subscribe failed: ${String(e)}`;
    }
  }

  function unmount(): void {
    if (_unlisten) {
      _unlisten();
      _unlisten = null;
    }
    _mounted = false;
  }

  return {
    get entries() {
      return entries;
    },
    get loading() {
      return loading;
    },
    get loadError() {
      return loadError;
    },
    get lastEvent() {
      return lastEvent;
    },
    refresh,
    fetchOne,
    submitReview,
    mount,
    unmount,
  };
}

export const mediaValidation = createMediaValidation();
