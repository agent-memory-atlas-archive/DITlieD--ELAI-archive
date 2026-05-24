// ============================================================
// feature_flags.svelte.ts — compile-time + runtime feature gates
//
// B-3 #27/#28 + V2 rebaseline. The Tauri shell exposes
// `get_feature_flags()` which returns:
//   - autonomous_mode  (cfg!(feature="autonomous-mode"))    — Phase 4G
//   - self_modification(cfg!(feature="self-modification"))  — Phase 4H
//   - dashboard_offline(env::var("ELAI_DASHBOARD_OFFLINE")) — V2 rebaseline
//
// The dashboard queries once on bootstrap (the values never change for the
// lifetime of the shell process) and exposes a $state-backed snapshot so
// consumers can guard panel renders. ProposalsPanel + AutoAdvanceStats
// mount only when their respective flag is true; without the gates, both
// panels rendered unconditionally even when their producers (B-22 / B-23)
// were not yet shipped.
// ============================================================

import type { FeatureFlags } from '../api/types';
import { get_feature_flags, logStub } from '../api/stub';

function createFeatureFlags() {
  // Default closed: panels stay hidden until the flag query resolves.
  // Pure-browser mode also lands here permanently because stub.ts returns
  // the all-false fallback when no Tauri shell is present.
  let flags = $state<FeatureFlags>({
    autonomous_mode: false,
    self_modification: false,
    dashboard_offline: false,
  });
  let loaded = $state(false);

  async function bootstrap(): Promise<void> {
    if (loaded) return;
    try {
      flags = await get_feature_flags();
      loaded = true;
      // Hydrate the window-level offline signal so stub.ts's tryInvoke
      // wrapper can short-circuit subsequent commands without re-querying.
      if (typeof window !== 'undefined' && flags.dashboard_offline) {
        (window as any).__ELAI_DASHBOARD_OFFLINE__ = true;
      }
      logStub('featureFlags.bootstrap', flags as unknown as Record<string, unknown>);
    } catch (err) {
      console.debug('[featureFlags] bootstrap failed', err);
    }
  }

  return {
    get autonomous_mode() { return flags.autonomous_mode; },
    get self_modification() { return flags.self_modification; },
    get dashboard_offline() { return flags.dashboard_offline; },
    get loaded() { return loaded; },
    bootstrap,
  };
}

export const featureFlags = createFeatureFlags();
