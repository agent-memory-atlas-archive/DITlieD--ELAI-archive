<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { workspace } from '../state/workspace.svelte';
  import { commands } from '../state/commands.svelte';
  import { installShortcuts } from '../state/shortcuts';
  import type { WindowType } from '../api/types';
  import WindowControls from '../chrome/WindowControls.svelte';

  interface Props {
    sidebar: Snippet;
    /** H-11: optional explorer rail content (FileExplorer.svelte). When absent
     *  the explorer column has 0 width (--explorer-width defaults to 0px). */
    explorer?: Snippet;
    primaryPanel: Snippet;
    divider: Snippet;
    secondaryPanel: Snippet;
    rail: Snippet;
    statusBar: Snippet;
    overlay?: Snippet;
    children?: Snippet;
  }

  let { sidebar, explorer, primaryPanel, divider, secondaryPanel, rail, statusBar, overlay }: Props = $props();

  let shellEl = $state<HTMLElement | undefined>(undefined);
  let dropStripHover = $state(false);

  // Drop-strip accepts BOTH drag flavors when the secondary panel is collapsed:
  //   - text/elai-tab               → MOVE existing tab into secondary
  //   - application/x-elai-tab-type → OPEN new tab in secondary
  // Both auto-uncollapse the secondary panel as part of the operation.
  const MIME_TAB = 'text/elai-tab';
  const MIME_TAB_TYPE = 'application/x-elai-tab-type';
  const VALID_TYPES = new Set(['command', 'plan', 'activity', 'review', 'monitor', 'settings', 'graph', 'idea_framing', 'audit_timeline', 'explore']);

  function dropStripAcceptsDrag(e: DragEvent): boolean {
    if (!e.dataTransfer) return false;
    const types = e.dataTransfer.types;
    return types && (Array.from(types).includes(MIME_TAB) || Array.from(types).includes(MIME_TAB_TYPE));
  }

  function onDropStripDragEnter(e: DragEvent) {
    if (!workspace.tabDragging) return;
    if (!dropStripAcceptsDrag(e)) return;
    e.preventDefault();
    dropStripHover = true;
  }

  function onDropStripDragOver(e: DragEvent) {
    if (!workspace.tabDragging) return;
    if (!dropStripAcceptsDrag(e)) return;
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = Array.from(e.dataTransfer.types).includes(MIME_TAB_TYPE) ? 'copy' : 'move';
    }
  }

  function onDropStripDragLeave() {
    dropStripHover = false;
  }

  function onDropStripDrop(e: DragEvent) {
    e.preventDefault();
    dropStripHover = false;
    if (!e.dataTransfer) return;

    // Sidebar-icon drop → OPEN new tab in secondary (auto-expands collapsed secondary).
    const newType = e.dataTransfer.getData(MIME_TAB_TYPE);
    if (newType && VALID_TYPES.has(newType)) {
      // openTab itself uncollapses secondary when target='secondary'.
      workspace.openTab('secondary', newType as WindowType);
      return;
    }

    // Existing-tab drop → MOVE to secondary.
    const raw = e.dataTransfer.getData(MIME_TAB);
    if (!raw) return;
    try {
      const data = JSON.parse(raw) as { tabId: string; fromPanel: string };
      if (typeof data.tabId !== 'string') return;
      // Make sure secondary is visible before moveTab fires.
      workspace.expandSecondary();
      workspace.moveTab(data.tabId, 'secondary');
    } catch {
      // malformed drag payload — ignore
    }
  }

  function onWindowResize() {
    if (shellEl) {
      const total = shellEl.querySelector<HTMLElement>('.panels-area')?.offsetWidth ?? 0;
      if (total > 0) {
        const px = Math.floor(total * workspace.panelSplit);
        shellEl.style.setProperty('--panel-split', `${px}px`);
      }
    }
  }

  onMount(() => {
    onWindowResize();
    window.addEventListener('resize', onWindowResize, { passive: true });

    const uninstallShortcuts = installShortcuts();

    // Esc closes overlays in priority order: newTabPicker → drawer → commandPalette
    function handleEscape(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (workspace.newTabPickerOpen) {
          workspace.closeNewTabPicker();
        } else if (workspace.activeDrawer !== null) {
          workspace.closeDrawer();
        } else if (workspace.commandPaletteOpen) {
          workspace.closeCommandPalette();
        }
      }
    }

    // Ctrl+. — focus command window and open slash palette
    function handleCtrlDot(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === '.') {
        e.preventDefault();
        workspace.setPrimaryWindow('command');
        commands.openSlash();
      }
    }

    window.addEventListener('keydown', handleEscape);
    window.addEventListener('keydown', handleCtrlDot);

    return () => {
      window.removeEventListener('resize', onWindowResize);
      window.removeEventListener('keydown', handleEscape);
      window.removeEventListener('keydown', handleCtrlDot);
      uninstallShortcuts();
    };
  });
</script>

<div
  class="app-shell anim-enter-fade"
  bind:this={shellEl}
  style="--panel-split: 50%; --explorer-width: {workspace.explorerWidth}px"
>
  <!-- Frameless titlebar chrome (decorations:false). Window controls
       float in the top-right corner with a guaranteed-free drag zone to
       their left; a second short drag strip sits over the top-left
       sidebar header gap. These two zones keep the window movable +
       closable on the workspace WITHOUT overlapping the 32px panel
       tab-bars (which own their own clicks). Renders nothing outside
       the Tauri shell. -->
  <div class="winctl-host">
    <span class="winctl-drag" data-tauri-drag-region aria-hidden="true"></span>
    <WindowControls />
  </div>

  <div class="sidebar-col">
    {@render sidebar()}
  </div>

  {#if explorer && workspace.explorerWidth > 0}
    <div class="explorer-col">
      {@render explorer()}
    </div>
  {/if}

  <div class="panels-area">
    {@render primaryPanel()}
    {@render divider()}
    {@render secondaryPanel()}

    {#if workspace.secondaryCollapsed && workspace.tabDragging}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drop-strip"
        class:drop-strip--hover={dropStripHover}
        aria-label="Drop tab here to dock as secondary panel"
        ondragenter={onDropStripDragEnter}
        ondragover={onDropStripDragOver}
        ondragleave={onDropStripDragLeave}
        ondrop={onDropStripDrop}
      ></div>
    {/if}
  </div>

  <div class="rail-col">
    {@render rail()}
  </div>

  <div class="statusbar-row">
    {@render statusBar()}
  </div>

  {#if overlay}
    <div class="overlay-host">
      {@render overlay()}
    </div>
  {/if}
</div>

<style>
  .app-shell {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    display: grid;
    /* H-11: added explorer rail column (column 2). Column 1 = 48px Sidebar
     * (C-6 preserved). Column 2 = explorer rail (--explorer-width CSS var,
     * default 0 = hidden). Column 3 = main panels area. Column 4 = right rail.
     * SEPARATE from panelSplit; driven by workspace.explorerWidth.
     * Sidebar.svelte grid-column remains 1; panels-area shifts from 2 -> 3. */
    grid-template-columns: 48px var(--explorer-width, 0px) 1fr 0; /* C-6 + H-11 */
    grid-template-rows: 1fr 22px;
    background: var(--bg-void);
    overflow: hidden;
  }

  .sidebar-col {
    grid-column: 1;
    grid-row: 1 / 3;
    display: flex;
    flex-direction: column;
    background: var(--bg-void);
    border-right: 1px solid var(--border-subtle);
    z-index: 10;
  }

  /* H-11: explorer rail slot (column 2). Hidden when --explorer-width is 0. */
  .explorer-col {
    grid-column: 2;
    grid-row: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-void);
    border-right: 1px solid var(--border-subtle);
    min-width: 0;
  }

  .panels-area {
    grid-column: 3; /* H-11: shifted from 2 -> 3 to make room for explorer rail */
    grid-row: 1;
    display: flex;
    flex-direction: row;
    overflow: hidden;
    background: var(--bg-base);
    min-width: 0;
    position: relative;
  }

  /*
   * Collapsed-secondary drop target.
   * Visible only while a tab drag is in progress (`workspace.tabDragging`).
   * Idle: 64px wide tinted strip with hint text — wide enough to be
   * discoverable. 8px was the prior value and users couldn't find it.
   * Hover: expands to 120px with brighter teal border + soft fill.
   */
  .drop-strip {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 64px;
    border-left: 2px dashed var(--teal-line);
    background: var(--teal-soft);
    opacity: 0.55;
    pointer-events: auto;
    z-index: 8;
    transition: width 140ms var(--ease-std), background 140ms var(--ease-std), border-color 140ms var(--ease-std), opacity 140ms var(--ease-std);
    cursor: copy;
  }

  .drop-strip::after {
    content: "Drop here";
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%) rotate(-90deg);
    transform-origin: center;
    font-family: var(--font-sans, sans-serif);
    font-size: 10px;
    letter-spacing: 0.15em;
    color: var(--teal);
    white-space: nowrap;
    pointer-events: none;
    text-transform: uppercase;
  }

  .drop-strip:hover,
  .drop-strip--hover {
    width: 120px;
    opacity: 1;
    background: var(--teal-soft);
    border-left-color: var(--teal);
  }

  .rail-col {
    display: none;
  }

  .statusbar-row {
    grid-column: 2 / 5; /* H-11: extended to cover explorer col + panels + right rail */
    grid-row: 2;
    display: flex;
    align-items: stretch;
  }

  .overlay-host {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 100;
  }

  /* Frameless window controls cluster — top-right corner, above the
     32px panel tab-bars. The drag strip to the controls' left is a
     reliably-free grab zone for moving the window on the workspace. */
  .winctl-host {
    position: fixed;
    top: 0;
    right: 0;
    height: 32px;
    display: flex;
    align-items: center;
    z-index: 60; /* above tabbars (panels-area content) + sidebar (10) */
  }
  .winctl-drag {
    /* ~120px guaranteed-draggable gutter left of the controls */
    width: 120px;
    height: 32px;
    align-self: stretch;
  }

</style>
