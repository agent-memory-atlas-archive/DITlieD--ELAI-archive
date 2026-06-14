<script lang="ts">
  // MenuFlyout.svelte — the ZERO-WIDTH flyout that expands LEFT->RIGHT over
  // the workspace when an IconRail icon is clicked (DESIGN §8). Scrim behind,
  // ~220ms slide, hidden while working, never a permanent content rail.
  //
  // Holds three sections (brief §5):
  //   MENU      — project intelligence + sessions/branches + receipts + a
  //               SETTINGS entry that opens the modal.
  //   MONITORS  — the legacy panel long-tail, collapsed. Each row opens the
  //               EXISTING drawer (workspace.setActiveDrawer) so the ~90
  //               panels stay MOUNTED + working (Drawer overlay in Workspace).
  //   FILES     — the H-11 file tree (FileExplorer + read_dir wiring).
  //
  // State coverage (DESIGN §9): files section shows loading / empty / error /
  // disconnected via FileExplorer's own states.
  import { shell } from '../../state/shell.svelte';
  import { workspace } from '../../state/workspace.svelte';
  import { session } from '../../state/session.svelte';
  import type { DrawerKey } from '../../api/types';
  import FileExplorer from '../../windows/FileExplorer.svelte';

  interface Props {
    /** When true, scroll to the FILES section on open. */
    focusFiles?: boolean;
  }
  let { focusFiles = false }: Props = $props();

  // Curated monitor long-tail. Each entry opens its EXISTING drawer panel.
  // (The full drawer set lives in Rail.svelte; this surfaces the common ones
  //  plus the trust/telemetry cluster the redesign brief calls out.)
  interface MonRow { key: DrawerKey; label: string; hot?: boolean }
  const monitorRows: MonRow[] = [
    { key: 'project-intelligence', label: 'project intelligence', hot: true },
    { key: 'providers',            label: 'runtime / backends' },
    { key: 'reasoning',            label: 'reasoning / GRC' },
    { key: 'orchestration',        label: 'orchestration' },
    { key: 'rules',                label: 'rules / authz' },
    { key: 'research',             label: 'research' },
    { key: 'knowledge',            label: 'knowledge / retrieval' },
    { key: 'memory-facts',         label: 'memory facts' },
    { key: 'cosmos',               label: 'cosmos artifact tier' },
    { key: 'web-quality',          label: 'web quality' },
    { key: 'anti-staleness',       label: 'anti-staleness' },
    { key: 'tools-hooks',          label: 'tools & hooks' },
    { key: 'vibegate',             label: 'vibegate / security' },
    { key: 'session-archive',      label: 'session archive' },
  ];

  let filesEl = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    // When asked to focus files, scroll the section into view once open.
    if (shell.flyoutOpen && focusFiles && filesEl) {
      requestAnimationFrame(() => filesEl?.scrollIntoView({ block: 'start' }));
    }
  });

  function openDrawer(key: DrawerKey) {
    workspace.setActiveDrawer(key);
    shell.closeFlyout();
  }

  let rootPath = $derived(session.currentProject?.path ?? '');
</script>

<!-- scrim behind the flyout -->
<div
  class="scrim"
  class:scrim--on={shell.flyoutOpen}
  onclick={() => shell.closeFlyout()}
  role="presentation"
  aria-hidden="true"
></div>

<nav
  class="flyout"
  class:flyout--open={shell.flyoutOpen}
  aria-label="Menu and files"
  aria-hidden={!shell.flyoutOpen}
>
  <div class="flyout-scroll">
    <div class="fsec">MENU</div>
    <button class="fitem" onclick={() => openDrawer('project-intelligence')}>
      <span>project intelligence</span>
    </button>
    <button class="fitem" onclick={() => openDrawer('orchestration')}>
      <span>sessions / branches</span>
      <span class="k">{session.sessionId}</span>
    </button>
    <button class="fitem" onclick={() => openDrawer('rules')}>
      <span>receipts / RRA</span>
      <span class="k">elai rra show</span>
    </button>
    <button class="fitem" onclick={() => shell.openSettings()}>
      <span>settings</span>
      <span class="k">⌘,</span>
    </button>

    <div class="fsec">
      MONITORS <span class="fsec-note">collapsed</span>
    </div>
    {#each monitorRows as row (row.key)}
      <button class="mon" class:mon--hot={row.hot} onclick={() => openDrawer(row.key)}>
        {row.label}
      </button>
    {/each}

    <div class="fsec" bind:this={filesEl}>FILES</div>
    <div class="files-host">
      {#if rootPath}
        <FileExplorer {rootPath} />
      {:else}
        <div class="files-empty">no project open</div>
      {/if}
    </div>
  </div>
</nav>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: var(--z-shell-overlay, 40);
    background: rgba(0, 0, 0, 0.35);
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--dur-slow) var(--ease-out);
  }
  .scrim--on { opacity: 1; pointer-events: auto; }

  .flyout {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 300px;
    z-index: var(--z-shell-flyout, 50);
    background: var(--panel);
    border-right: 1px solid var(--line);
    transform: translateX(-100%);
    transition: transform var(--dur-slow) var(--ease-out);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .flyout--open {
    transform: translateX(0);
    box-shadow: 18px 0 50px rgba(0, 0, 0, 0.5);
  }

  .flyout-scroll {
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 50px 14px 18px;
    display: flex;
    flex-direction: column;
  }

  .fsec {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.22em;
    color: var(--dim);
    margin: 18px 0 8px;
  }
  .fsec::before {
    content: "";
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--d-teal-bright);
    flex: none;
  }
  .fsec:first-child { margin-top: 0; }
  .fsec-note { margin-left: auto; color: var(--ghost); }

  .fitem {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    width: 100%;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text);
    padding: 7px 9px;
    background: none;
    border: none;
    border-radius: var(--r-md);
    cursor: pointer;
    text-align: left;
    transition: background var(--dur-fast) var(--ease-out);
  }
  .fitem:hover { background: var(--d-teal-ghost); }
  .fitem:focus-visible { box-shadow: inset 0 0 0 1px var(--d-teal); }
  .fitem .k { color: var(--ghost); font-size: 9px; flex: none; }

  /* I3: monitor nav rows use center->out underline (DESIGN §13), not the dot.
     The dot is for STATIC section labels (.fsec::before) only (§4). */
  .mon {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--dim);
    padding: 5px 9px;
    background: none;
    border: none;
    border-radius: var(--r-sm);
    cursor: pointer;
    text-align: left;
    outline: none;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .mon::after {
    content: '';
    position: absolute;
    left: 9px;
    right: 9px;
    bottom: 2px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .mon:hover { color: var(--text); }
  .mon:hover::after { transform: scaleX(1); }
  .mon--hot { color: var(--read); }
  .mon--hot::after { transform: scaleX(1); }
  .mon:focus-visible { box-shadow: inset 0 0 0 1px var(--d-teal); }

  .files-host {
    margin-top: 4px;
    min-height: 160px;
    max-height: 320px;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    overflow: hidden;
  }
  .files-empty {
    padding: 16px 10px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--dim);
  }

  @media (prefers-reduced-motion: reduce) {
    .scrim, .flyout { transition: none; }
    /* underline appears instantly (no scaleX animation) */
    .mon::after { transition: none; }
  }
</style>
