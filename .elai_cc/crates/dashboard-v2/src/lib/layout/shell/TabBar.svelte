<script lang="ts">
  // TabBar.svelte — the 4-question-tab primary nav (DESIGN §8 / brief §4).
  // Shared Wordmark + back-chevron at left, tabs center, theme toggle +
  // window-controls slot at right. Height 48px (matches ProjectSelector §13).
  //
  // Interactive nav uses the center->out underline idiom (DESIGN §13, §14
  // open-item resolved): no dot, no box. The dot-prefix is for STATIC section
  // labels only (§4). Running n/m segmented fraction bar at the right (DESIGN
  // §6: progress is always an explicit fraction, never a lone percentage).
  import { shell, SHELL_TABS, TAB_LABEL } from '../../state/shell.svelte';
  import { plan } from '../../state/plan.svelte';
  import { session } from '../../state/session.svelte';
  import type { DecompNode } from '../../api/types';
  import BrandCluster from './BrandCluster.svelte';

  // Count leaf nodes by status to build the n/m running fraction.
  function countLeaves(node: DecompNode | null | undefined, acc: { done: number; total: number; active: boolean }) {
    if (!node) return;
    const kids = node.children ?? [];
    if (kids.length === 0) {
      // leaf
      if (node.id === 'empty') return;
      acc.total += 1;
      const st = String(node.status);
      if (st === 'done' || st === 'complete' || st === 'completed' || st === 'pass' || st === 'passed') acc.done += 1;
      if (st === 'active' || st === 'executing' || st === 'running' || st === 'in_progress') acc.active = true;
    } else {
      for (const k of kids) countLeaves(k, acc);
    }
  }

  let frac = $derived.by(() => {
    const acc = { done: 0, total: 0, active: false };
    countLeaves(plan.tree, acc);
    return acc;
  });

  // Up to 14 segments in the running bar (mockup uses 14). Each segment is
  // filled (done), active (the current one, pulsing), or empty.
  const SEG_COUNT = 14;
  let segments = $derived.by(() => {
    const { done, total, active } = frac;
    if (total === 0) return [] as ('f' | 'a' | 'e')[];
    const filled = Math.round((done / total) * SEG_COUNT);
    const out: ('f' | 'a' | 'e')[] = [];
    for (let i = 0; i < SEG_COUNT; i++) {
      if (i < filled) out.push('f');
      else if (i === filled && active) out.push('a');
      else out.push('e');
    }
    return out;
  });
</script>

<header class="tabbar" data-tauri-drag-region>
  <!-- shared brand lockup: back chevron + ELAI text wordmark (identical
       to the project-selector topbar — DESIGN §16) -->
  <BrandCluster onBack={() => session.openSelector()} backLabel="Back to project selection" />

  <div class="tabs" role="tablist" aria-label="Primary navigation">
    {#each SHELL_TABS as tab (tab)}
      <button
        class="tab"
        class:tab--on={shell.activeTab === tab}
        role="tab"
        aria-selected={shell.activeTab === tab}
        onclick={() => shell.setTab(tab)}
      >
        {TAB_LABEL[tab]}
      </button>
    {/each}
  </div>

  <div class="tab-right">
    {#if frac.total > 0}
      <div class="runbar" aria-label="Progress: {frac.done} of {frac.total} steps">
        <span class="run-label">{frac.active ? 'RUNNING' : 'PROGRESS'}</span>
        <span class="segbar" aria-hidden="true">
          {#each segments as s}
            <span class="seg" class:seg--f={s === 'f'} class:seg--a={s === 'a'}></span>
          {/each}
        </span>
        <span class="frac">{frac.done}/{frac.total}</span>
      </div>
    {/if}
    <span class="sid">{session.sessionId}</span>
    <!-- theme toggle + window controls live in WorkflowShell.winctl-host
         (the shared TopRightControls cluster, identical on all 3 screens) -->
  </div>
</header>

<style>
  .tabbar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    height: 100%;
    /* left inset matches the selector topbar (16px) so the shared
       BrandCluster sits at the identical x on both — DESIGN §16 */
    padding: 0 14px 0 16px;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
  }

  /* nav centered absolutely in the bar — the selector's mode-selector
     idiom (DESIGN §16): brand left, nav dead-center, status/controls right. */
  .tabs {
    position: absolute;
    left: 50%;
    top: 0;
    transform: translateX(-50%);
    height: 100%;
    display: flex;
    align-items: center;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  /* Tabs use center->out underline idiom (DESIGN §13/§14).
     The dot-prefix is reserved for STATIC section labels only (§4). */
  .tab {
    position: relative;
    display: flex;
    align-items: center;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    padding: 6px 13px;
    color: var(--dim);
    background: none;
    border: none;
    cursor: pointer;
    outline: none;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .tab::after {
    content: '';
    position: absolute;
    left: 8px;
    right: 8px;
    bottom: 4px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .tab:hover:not(.tab--on) {
    color: var(--text);
  }
  .tab:hover::after {
    transform: scaleX(1);
  }
  .tab--on {
    color: var(--d-teal-bright);
  }
  .tab--on::after {
    transform: scaleX(1);
  }
  .tab:focus-visible {
    box-shadow: inset 0 0 0 1px var(--d-teal);
    border-radius: var(--r-sm);
  }

  .tab-right {
    margin-left: auto;
    /* clear the fixed TopRightControls cluster (toggle + window controls)
       floating at right:20px in WorkflowShell.winctl-host */
    margin-right: 190px;
    display: flex;
    align-items: center;
    gap: 10px;
    -webkit-app-region: no-drag;
  }

  .runbar {
    display: flex;
    align-items: center;
    gap: 9px;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.14em;
    color: var(--dim);
  }
  .run-label { color: var(--dim); }
  .frac { color: var(--d-teal-bright); }

  .segbar { display: flex; gap: 2px; }
  .seg {
    width: 9px;
    height: 8px;
    border-radius: 1px;
    background: var(--ghost);
  }
  .seg--f { background: var(--d-teal-deep); }
  .seg--a {
    background: var(--d-teal-bright);
    animation: segpulse 1.6s ease-in-out infinite;
  }
  @keyframes segpulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.35; }
  }

  .sid {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.14em;
    color: var(--dim);
  }

  @media (prefers-reduced-motion: reduce) {
    .seg--a { animation: none; }
    .tab::after { transition: none; }
  }
</style>
