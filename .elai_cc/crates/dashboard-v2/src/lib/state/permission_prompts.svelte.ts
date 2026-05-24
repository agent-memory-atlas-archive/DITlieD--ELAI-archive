// E-33: permission-prompt store backing SnackBarPermission.svelte.
//
// Polls get_recent_actions for the most recent `intervention_request`
// SSB rows and surfaces them as PermissionPrompt entries above the
// input bar (mounted in WindowHost.svelte). The 5 dedicated SSB
// variants (ActivityRibbonInteracted / ConfidenceTagHovered /
// SkeletonReplaced / ReducedMotionActivated /
// WindowFooterModeChanged) are SCHEMA-DEFERRED per the binding
// decisions; until they land the store reuses the existing
// intervention_request channel as the canonical permission surface.
//
// Approve / reject / always-allow currently log via the stub
// dispatcher (no backend mutation), so a sidecar-offline state never
// crashes the snackbar. The id-keyed dedup prevents the same request
// reappearing across polls.

import * as stub from '../api/stub';
import { logStub } from '../api/stub';

const POLL_INTERVAL_MS = 5_000;
const MAX_VISIBLE = 4;

interface PermissionPrompt {
  id: string;
  title: string;
  detail?: string;
  tool?: string;
}

function createPermissionPrompts() {
  let prompts = $state<PermissionPrompt[]>([]);
  let dismissed = $state<Set<string>>(new Set<string>());
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let alive = false;

  function buildId(seq: number, ts: number): string {
    return `perm-${seq}-${ts}`;
  }

  async function refresh(): Promise<void> {
    try {
      const rows = await stub.get_recent_actions(20, 0);
      const next: PermissionPrompt[] = [];
      for (const row of rows) {
        if (row.action_type !== 'intervention_request') continue;
        const id = buildId(row.seq, row.timestamp);
        if (dismissed.has(id)) continue;
        next.push({
          id,
          title: row.summary || 'permission requested',
          detail: row.agent_id ? `from ${row.agent_id}` : undefined,
          tool: row.action_type,
        });
        if (next.length >= MAX_VISIBLE) break;
      }
      prompts = next;
    } catch (err) {
      console.debug('[permission_prompts] refresh failed', err);
    }
  }

  function dismiss(id: string): void {
    dismissed.add(id);
    prompts = prompts.filter((p) => p.id !== id);
  }

  return {
    get prompts() {
      return prompts;
    },
    bootstrap() {
      if (alive) return;
      alive = true;
      void refresh();
      pollTimer = setInterval(() => {
        void refresh();
      }, POLL_INTERVAL_MS);
      logStub('permission_prompts.bootstrap');
    },
    teardown() {
      if (pollTimer !== null) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
      alive = false;
    },
    approve(id: string): void {
      logStub('permission_prompts.approve', { id });
      dismiss(id);
    },
    reject(id: string): void {
      logStub('permission_prompts.reject', { id });
      dismiss(id);
    },
    alwaysAllow(id: string): void {
      logStub('permission_prompts.alwaysAllow', { id });
      dismiss(id);
    },
  };
}

export const permissionPrompts = createPermissionPrompts();
