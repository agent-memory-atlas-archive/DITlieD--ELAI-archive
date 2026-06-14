<script lang="ts">
  // ActivityWindow — thin host for the orbit visualizer.
  // The orbit IS the activity surface: it renders the live
  // workflow as a starmap. Mode (plan / map / research) is derived
  // from the FSM state. The default fixture is the live decomposition
  // tree from the plan store; mock fixtures load only on ?demo=1.

  import { onMount } from 'svelte';
  import OrbitCanvas from '$lib/orbit/OrbitCanvas.svelte';
  import { fsm } from '$lib/state/fsm.svelte';
  import { plan } from '$lib/state/plan.svelte';
  import { decompNodeToOrbitFixture } from '$lib/orbit/decompose-converter';
  import type { OrbitFixture, OrbitMode } from '$lib/orbit/types';
  // E-4 Task 13: InterventionControlBar overlay. Visible during
  // EXECUTING / AUDITING / RESEARCHING; auto-hidden otherwise via the
  // component's own visibility guard. Wires the /pause /redirect
  // /cancel verbs to the FSM through commands_e4 (wire-dark today).
  import InterventionControlBar from '$lib/components/InterventionControlBar.svelte';

  type FixtureKey = 'live' | 'small' | 'constellation' | 'plan' | 'map' | 'research';

  const FIXTURE_LABEL: Readonly<Record<FixtureKey, string>> = {
    live:          'live',
    small:         'task',
    constellation: 'constellation',
    plan:          'plan',
    research:      'research',
    map:           'codebase',
  };

  // For "demo" fixtures the orbit mode is intrinsic; for live + small/
  // constellation, the mode is derived from the dashboard's live FSM.
  const FIXTURE_MODE: Readonly<Record<FixtureKey, OrbitMode | undefined>> = {
    live:          undefined,
    small:         undefined,
    constellation: undefined,
    plan:          'plan',
    research:      'research',
    map:           'map',
  };

  // Demo gate: cycler only visible (and mock fixtures only loaded) when
  // the URL query contains ?demo=1. Removes mock-orbit weight from the
  // production bundle via vite code-splitting.
  const demoEnabled = typeof window !== 'undefined'
    && new URLSearchParams(window.location.search).get('demo') === '1';

  // Mock fixtures: lazy-loaded chunk. Production builds without ?demo=1
  // never resolve this import.
  let mockFixtures = $state<Partial<Record<FixtureKey, OrbitFixture>>>({});
  let mockLoaded = $state(false);

  onMount(() => {
    if (!demoEnabled) return;
    let cancelled = false;
    void import('$lib/api/mock-orbit')
      .then((m) => {
        if (cancelled) return;
        mockFixtures = {
          small:         m.ORBIT_FIXTURE_SMALL,
          constellation: m.ORBIT_FIXTURE_CONSTELLATION,
          plan:          m.ORBIT_FIXTURE_SMALL,
          research:      m.ORBIT_FIXTURE_RESEARCH,
          map:           m.ORBIT_FIXTURE_MAP,
        };
        mockLoaded = true;
      })
      .catch((err) => console.warn('[ActivityWindow] mock-orbit load failed', err));
    return () => { cancelled = true; };
  });

  let fixtureKey = $state<FixtureKey>('live');

  // Live fixture: convert the polled decomposition tree into the orbit shape.
  const liveBase = $derived(decompNodeToOrbitFixture(plan.tree, fsm.state));

  const liveFixture: OrbitFixture = $derived.by(() => {
    if (fixtureKey === 'live') return liveBase;
    const mock = mockFixtures[fixtureKey];
    // Mock keys before the chunk loads (or with ?demo=1 dropped post-load) fall back to live.
    if (!mock) return liveBase;
    return {
      ...mock,
      fsm: FIXTURE_MODE[fixtureKey] ? mock.fsm : fsm.state,
    };
  });
  const explicitMode = $derived(FIXTURE_MODE[fixtureKey]);

  function cycleFixture() {
    if (!demoEnabled) return;
    const order: FixtureKey[] = mockLoaded
      ? ['live', 'small', 'constellation', 'plan', 'map', 'research']
      : ['live'];
    const i = order.indexOf(fixtureKey);
    fixtureKey = order[(i + 1) % order.length];
  }
</script>

<div class="activity-host">
  <!-- top-left header removed: the concept name now lives only in the
       OrbitCaption below the canvas. Was overlapping with the FPS HUD
       and duplicating info the caption already shows. activeConcept
       reactive computation kept above for any future consumer. -->

  <!-- top-right: dev mode-toggle (only visible when ?demo=1). -->
  {#if demoEnabled}
    <button
      class="mode-toggle"
      onclick={cycleFixture}
      aria-label="cycle orbit fixture"
      title="dev: cycle orbit fixture"
    >
      <span class="mt-prefix">orbit</span>
      <span class="mt-key">{FIXTURE_LABEL[fixtureKey]}</span>
    </button>
  {/if}

  <!-- main orbit canvas -->
  <OrbitCanvas fixture={liveFixture} mode={explicitMode} />

  <!-- E-4 Task 13: bottom-centered intervention control bar. Self-hides
       outside EXECUTING / AUDITING / RESEARCHING via its own visibility
       guard, so this slot is empty most of the time. -->
  <div class="intervention-slot">
    <InterventionControlBar />
  </div>
</div>

<style>
  .activity-host {
    position: relative;
    width: 100%;
    height: 100%;
    background: var(--bg);
    overflow: hidden;
  }

  /* E-4 Task 13: control-bar slot. Centered bottom; pointer-events
     none on the wrapper so the orbit canvas underneath stays
     interactive when the bar is hidden. */
  .intervention-slot {
    position: absolute;
    bottom: var(--s-4, 16px);
    left: 50%;
    transform: translateX(-50%);
    z-index: 3;
    pointer-events: none;
  }
  .intervention-slot :global(.icb) {
    pointer-events: auto;
  }

  .mode-toggle {
    position: absolute;
    top: var(--s-3);
    right: var(--s-5);
    z-index: 3;
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    background: transparent;
    border: 1px solid var(--line);
    border-radius: var(--r-pill);
    padding: 3px var(--s-3);
    cursor: pointer;
    color: var(--read);
    font-family: var(--font-mono);
    opacity: 0;
    transition:
      opacity var(--dur-slow) var(--ease-out),
      border-color var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }
  .activity-host:hover .mode-toggle { opacity: 1; }
  .mode-toggle:focus-visible { opacity: 1; }
  .mode-toggle:hover {
    border-color: var(--d-teal-soft);
    color: var(--text);
  }
  .mt-prefix {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--dim);
  }
  .mt-key {
    font-size: 10px;
    color: var(--d-teal);
    text-transform: lowercase;
  }
</style>
