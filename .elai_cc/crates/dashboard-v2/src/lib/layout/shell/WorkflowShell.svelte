<script lang="ts">
  // WorkflowShell.svelte — the redesigned workflow shell (brief §5).
  // DEFAULTS + REDUCTION on the existing machinery: this is the NEW chrome
  // (4-question-tab bar + thin icon rail + left->right menu/file flyout +
  // settings modal + first-class chat beside the canvas + reskinned 22px
  // statusbar). It RE-HOMES the existing windows — every tab renders an
  // EXISTING WindowType through WindowHost, the chat is the existing
  // CommandWindow, settings wraps the existing SettingsWindow, and the
  // monitor long-tail opens the existing Drawer panels. No store, Tauri
  // command, or panel is removed.
  //
  // Grid:
  //   row 1  = TabBar (42px)
  //   row 2  = [ IconRail 48px | chat (var width) | splitter 5px | main pane 1fr ]
  //   row 3  = ShellStatusBar (22px)
  // Overlays: MenuFlyout (+scrim), SettingsModal, frameless WindowControls,
  //           plus the existing Workspace overlays (drawers/palette/recovery)
  //           passed in via the `overlay` snippet.
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { shell } from '../../state/shell.svelte';
  import TopRightControls from '../../chrome/TopRightControls.svelte';
  import WindowHost from '../WindowHost.svelte';
  import TabBar from './TabBar.svelte';
  import IconRail from './IconRail.svelte';
  import MenuFlyout from './MenuFlyout.svelte';
  import SettingsModal from './SettingsModal.svelte';
  import ShellStatusBar from './ShellStatusBar.svelte';

  interface Props {
    /** Existing Workspace overlays (drawers / command palette / recovery /
     *  remote-sidecar boot). Rendered above the workspace, below modal. */
    overlay?: Snippet;
  }
  let { overlay }: Props = $props();

  // Whether the flyout was opened focused on the FILES section.
  let flyoutFocusFiles = $state(false);

  function openMenu() {
    flyoutFocusFiles = false;
    shell.toggleFlyout();
  }
  function openFiles() {
    flyoutFocusFiles = true;
    if (!shell.flyoutOpen) shell.openFlyout();
  }

  // ---- chat/canvas splitter ----
  let shellBodyEl = $state<HTMLElement | undefined>(undefined);
  let dragging = $state(false);

  function onSplitterDown(e: PointerEvent) {
    dragging = true;
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    document.body.style.userSelect = 'none';
  }
  function onSplitterMove(e: PointerEvent) {
    if (!dragging || !shellBodyEl) return;
    const rect = shellBodyEl.getBoundingClientRect();
    // chat width = pointer X minus the 48px icon rail offset.
    const w = e.clientX - rect.left - 48;
    shell.setChatWidth(w);
  }
  function onSplitterUp(e: PointerEvent) {
    dragging = false;
    (e.target as HTMLElement).releasePointerCapture?.(e.pointerId);
    document.body.style.userSelect = '';
  }

  onMount(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (shell.handleEscape()) e.stopPropagation();
      }
      // ⌘, / Ctrl+, opens settings
      if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        shell.openSettings();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<div class="workflow-shell anim-panel-in">
  <!-- Frameless titlebar drag + window controls (kept working — brief §5 G).
       Drag strip left of the controls is a guaranteed-free grab zone. -->
  <div class="winctl-host">
    <TopRightControls />
  </div>

  <!-- row 1: 4-question-tab bar -->
  <div class="tabbar-row">
    <TabBar />
  </div>

  <!-- row 2: icon rail | chat | splitter | main pane -->
  <div class="shell-body" bind:this={shellBodyEl}>
    <IconRail onMenu={openMenu} onFiles={openFiles} />

    <aside class="chat-pane" style="width: {shell.chatWidth}px">
      <WindowHost type="command" />
    </aside>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="splitter"
      class:splitter--drag={dragging}
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize chat"
      onpointerdown={onSplitterDown}
      onpointermove={onSplitterMove}
      onpointerup={onSplitterUp}
    ></div>

    <main class="main-pane">
      {#key shell.activeTab}
        <WindowHost type={shell.activeWindow} />
      {/key}
    </main>
  </div>

  <!-- row 3: statusbar -->
  <div class="statusbar-row">
    <ShellStatusBar />
  </div>

  <!-- overlays -->
  <MenuFlyout focusFiles={flyoutFocusFiles} />
  <SettingsModal />

  {#if overlay}
    <div class="overlay-host">
      {@render overlay()}
    </div>
  {/if}
</div>

<style>
  .workflow-shell {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    display: grid;
    /* 67px top row so the fixed cluster (top:18px) seats centered — §16 */
    grid-template-rows: 67px 1fr 22px;
    background: var(--bg);
    overflow: hidden;
  }

  .tabbar-row {
    grid-row: 1;
    min-width: 0;
    /* full-width bar (so the hairline border spans edge-to-edge and the
       centered tabs land on the true window center); the fixed
       TopRightControls cluster floats over the right edge and TabBar's
       .tab-right reserves margin to clear it. */
  }

  .shell-body {
    grid-row: 2;
    display: flex;
    flex-direction: row;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  .chat-pane {
    flex: none;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border-right: 1px solid var(--line);
    background: var(--bg);
    overflow: hidden;
  }

  .splitter {
    flex: none;
    width: 5px;
    cursor: col-resize;
    background: var(--line);
    transition: background var(--dur-fast) var(--ease-out);
  }
  .splitter:hover,
  .splitter--drag { background: var(--d-teal-soft); }

  .main-pane {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .statusbar-row {
    grid-row: 3;
    display: flex;
    align-items: stretch;
  }

  .winctl-host {
    /* the shared top-right cluster, fixed at the same viewport coordinates
       as Intro + selector (top:18px, right:20px) — DESIGN §16 */
    position: fixed;
    top: 18px;
    right: 20px;
    display: flex;
    align-items: center;
    z-index: var(--z-shell-modal, 80);
  }

  .overlay-host {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: var(--z-shell-overlay, 40);
  }

  @media (prefers-reduced-motion: reduce) {
    .splitter { transition: none; }
  }
</style>
