// ============================================================
// fsm.svelte.ts — FSM state + current step + progress
//
// Phase B wiring: drop the mock FSM_SNAPSHOT seed. Initial state
// is IDLE/empty; bootstrap() polls get_fsm_state() every 5s and
// replaces local state with whatever the backend returns.
// ============================================================

import type { FsmState, FsmTransition } from '../api/types';
import { get_fsm_state, logStub } from '../api/stub';

const POLL_INTERVAL_MS = 5_000;

function createFsm() {
  let state = $state<FsmState>('IDLE');
  let currentStepId = $state<string>('');
  let progress = $state<number>(0);
  let transitionHistory = $state<FsmTransition[]>([]);
  let agentSubstate = $state<string>('');

  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let alive = false;

  async function refresh() {
    try {
      const snap = await get_fsm_state();
      state = snap.state;
      currentStepId = snap.current_step_id ?? '';
      progress = typeof snap.progress === 'number' ? snap.progress : 0;
      transitionHistory = Array.isArray(snap.history) ? [...snap.history] : [];
      agentSubstate = snap.agent_substate ?? '';
    } catch (err) {
      // Render last-known state; don't blow up the dashboard.
      console.debug('[fsm] refresh failed', err);
    }
  }

  return {
    get state() { return state; },
    get currentStepId() { return currentStepId; },
    get progress() { return progress; },
    get transitionHistory() { return transitionHistory; },
    get agentSubstate() { return agentSubstate; },

    /**
     * Start polling the backend. Called from Workspace.svelte onMount.
     * Idempotent — subsequent calls are no-ops while the timer is running.
     */
    bootstrap() {
      if (alive) return;
      alive = true;
      void refresh();
      pollTimer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
      logStub('fsm.bootstrap');
    },

    teardown() {
      if (pollTimer !== null) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
      alive = false;
    },

    transition(to: FsmState) {
      const from = state;
      transitionHistory = [...transitionHistory, { from, to, ts: Date.now(), trigger: 'manual' }];
      state = to;
      logStub('fsm.transition', { from, to });
    },

    setStep(id: string) {
      currentStepId = id;
    },

    setProgress(p: number) {
      progress = Math.max(0, Math.min(1, p));
    },

    setAgentSubstate(sub: string) {
      agentSubstate = sub;
    },

    /** No-op tick; retained so existing callers compile. */
    tick() {},
  };
}

export const fsm = createFsm();
