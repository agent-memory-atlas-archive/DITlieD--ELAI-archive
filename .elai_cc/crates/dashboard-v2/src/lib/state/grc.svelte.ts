// E-33: GRC confidence stream.
//
// Polls get_recent_actions for the most recent `reasoning_trace`
// SSB row, decodes the `confidence` field on the payload via
// get_action_detail, and exposes a numeric 0..1 value that
// ConfidenceTag can render. The three-state Rust enum
// (Verified / High confidence / Uncertain) maps to numeric bands
// matching the ConfidenceTag::classify boundaries in
// crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte:
//   Verified        -> 0.95 (-> 'verified')
//   High confidence -> 0.70 (-> 'ok')
//   Uncertain       -> 0.30 (-> 'low')
//   unknown / null  -> 0.10 (-> 'needs-evidence')
//
// Polled every 5s, same cadence as fsm.svelte.ts. Errors are
// silent — the orbit + dashboard stay rendering on the last known
// value (or the default 0.10 needs-evidence) so a sidecar offline
// state never blanks the UI.

import * as stub from '../api/stub';
import { logStub } from '../api/stub';

const POLL_INTERVAL_MS = 5_000;

type GrcConfidenceLabel = 'Verified' | 'High confidence' | 'Uncertain';

function labelToValue(label: GrcConfidenceLabel | string | null | undefined): number {
  switch (label) {
    case 'Verified':
      return 0.95;
    case 'High confidence':
      return 0.70;
    case 'Uncertain':
      return 0.30;
    default:
      return 0.10;
  }
}

function createGrc() {
  let confidence = $state<number>(0.10);
  let stepLabel = $state<string>('');
  let lastSeq = $state<number>(0);
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let alive = false;

  async function refresh(): Promise<void> {
    try {
      const rows = await stub.get_recent_actions(20, 0);
      const trace = rows.find((r) => r.action_type === 'reasoning_trace');
      if (!trace) return;
      if (trace.seq === lastSeq) return;
      const detail = await stub.get_action_detail(trace.seq);
      if (!detail.known) return;
      const payload = detail.payload_json as
        | { kind?: string; step_type?: string; confidence?: GrcConfidenceLabel }
        | null
        | undefined;
      if (!payload || payload.kind !== 'reasoning') return;
      confidence = labelToValue(payload.confidence);
      stepLabel = payload.step_type ?? '';
      lastSeq = trace.seq;
    } catch (err) {
      // sidecar offline / IPC error — keep the last known value.
      console.debug('[grc] refresh failed', err);
    }
  }

  return {
    /** Numeric 0..1 confidence value safe to pass to ConfidenceTag. */
    get confidence() {
      return confidence;
    },
    /** GRC step name (observe / hypothesize / ...) of the most recent trace. */
    get stepLabel() {
      return stepLabel;
    },
    get lastSeq() {
      return lastSeq;
    },
    bootstrap() {
      if (alive) return;
      alive = true;
      void refresh();
      pollTimer = setInterval(() => {
        void refresh();
      }, POLL_INTERVAL_MS);
      logStub('grc.bootstrap');
    },
    teardown() {
      if (pollTimer !== null) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
      alive = false;
    },
  };
}

export const grc = createGrc();
