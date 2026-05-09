// reindex.svelte.ts — Tauri "reindex-progress" event store
//
// Lives at the App root so the listener registers BEFORE any reindex_now()
// fire-and-forget call from ProjectSelector. The StatusBar (mounted only on
// the workspace screen) reads reactive state from this store; it does not
// own the Tauri channel.

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type ReindexCounts = {
  dkg_files: number;
  dkg_nodes: number;
  dkg_edges: number;
  bm25_files: number;
  bm25_chunks: number;
  profile_loaded?: boolean;
};

export type ReindexPayload =
  | { status: 'started' }
  | ({ status: 'done' } & ReindexCounts)
  | { status: 'error'; error: string };

export type ReindexUiStatus = 'idle' | 'running' | 'done' | 'error';

const FADE_AFTER_DONE_MS = 3000;

function createReindexStore() {
  let status = $state<ReindexUiStatus>('idle');
  let counts = $state<ReindexCounts | null>(null);
  let error = $state<string | null>(null);

  let alive = false;
  let unlisten: UnlistenFn | null = null;
  let cancelled = false;
  let fadeTimer: number | null = null;
  // Caches the in-flight listen() Promise so callers can `await ready()` before
  // firing the first reindex_now(). Resolves when the listener is registered
  // (or rejection is swallowed and we degrade to listener-less behavior).
  let readyPromise: Promise<void> | null = null;

  function clearFadeTimer() {
    if (fadeTimer !== null) {
      window.clearTimeout(fadeTimer);
      fadeTimer = null;
    }
  }

  function applyPayload(p: ReindexPayload) {
    if (p.status === 'started') {
      status = 'running';
      counts = null;
      error = null;
      clearFadeTimer();
    } else if (p.status === 'done') {
      status = 'done';
      counts = {
        dkg_files: p.dkg_files,
        dkg_nodes: p.dkg_nodes,
        dkg_edges: p.dkg_edges,
        bm25_files: p.bm25_files,
        bm25_chunks: p.bm25_chunks,
        profile_loaded: p.profile_loaded,
      };
      error = null;
      clearFadeTimer();
      fadeTimer = window.setTimeout(() => { status = 'idle'; }, FADE_AFTER_DONE_MS);
    } else if (p.status === 'error') {
      status = 'error';
      counts = null;
      error = p.error;
      clearFadeTimer();
    }
  }

  return {
    get status() { return status; },
    get counts() { return counts; },
    get error() { return error; },

    bootstrap() {
      if (alive) return;
      alive = true;
      cancelled = false;
      readyPromise = listen<ReindexPayload>('reindex-progress', (event) => applyPayload(event.payload))
        .then((un) => {
          if (cancelled) {
            // Listener resolved after teardown; clean up immediately.
            try { un(); } catch (err) { console.warn('[reindex] late unlisten failed', err); }
            return;
          }
          unlisten = un;
        })
        .catch((err) => {
          // Degrade to listener-less mode — better than blocking the UI on a
          // promise that will never resolve.
          console.warn('[reindex] listen() failed', err);
        });
    },

    /** Resolves when the reindex-progress listener has been registered (or
     *  failed and degraded). Callers MUST await this before firing the first
     *  reindex_now() to avoid missing the 'started' event. */
    ready(): Promise<void> {
      // No bootstrap call yet — return resolved Promise so callers don't hang.
      // (App.svelte calls bootstrap() during onMount which runs before any
      // ProjectSelector openProject; this is the defensive branch.)
      if (readyPromise === null) return Promise.resolve();
      return readyPromise;
    },

    teardown() {
      if (!alive) return;
      alive = false;
      cancelled = true;
      if (unlisten) {
        try { unlisten(); } catch (err) { console.warn('[reindex] unlisten failed', err); }
        unlisten = null;
      }
      clearFadeTimer();
    },
  };
}

export const reindex = createReindexStore();
