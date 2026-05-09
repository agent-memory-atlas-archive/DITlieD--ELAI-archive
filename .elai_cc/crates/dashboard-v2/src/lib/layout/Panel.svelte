<script lang="ts">
  import { X, Plus } from '@lucide/svelte';
  import WindowHost from './WindowHost.svelte';
  import { workspace } from '../state/workspace.svelte';

  type WindowType = 'command' | 'plan' | 'activity' | 'review' | 'explore' | 'monitor' | 'settings';

  interface TabDef {
    id: string;
    type: WindowType;
    title: string;
  }

  interface Props {
    panelId: 'primary' | 'secondary';
    tabs: TabDef[];
    activeTabId: string;
    onTabSelect: (tabId: string) => void;
    onTabClose: (tabId: string) => void;
    onNewTab: (panelId: 'primary' | 'secondary', type: WindowType) => void;
  }

  let { panelId, tabs, activeTabId, onTabSelect, onTabClose, onNewTab }: Props = $props();

  const WINDOW_TYPES: WindowType[] = ['command', 'plan', 'activity', 'review', 'explore', 'monitor', 'settings'];
  const WINDOW_LABELS: Record<WindowType, string> = {
    command:  'Command',
    plan:     'Plan',
    activity: 'Activity',
    review:   'Review',
    explore:  'Explore',
    monitor:  'Monitor',
    settings: 'Settings',
  };

  let dragOverTabId = $state<string | null>(null);
  let bodyDragHover = $state(false);

  // The new-tab picker is controlled by workspace state so the same menu logic
  // works for both panels and so closing one panel's picker doesn't leave a
  // stale `showNewTabMenu` flag behind.
  let pickerOpenHere = $derived(
    workspace.newTabPickerOpen && workspace.newTabPickerTarget === panelId
  );

  // Right-click context menu for tabs
  let ctxMenu = $state<{ tabId: string; x: number; y: number } | null>(null);

  function openContextMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    e.stopPropagation();
    // Clamp to viewport — assume menu max ~180x68px.
    const MENU_W = 180;
    const MENU_H = 72;
    const x = Math.min(e.clientX, window.innerWidth - MENU_W - 4);
    const y = Math.min(e.clientY, window.innerHeight - MENU_H - 4);
    ctxMenu = { tabId, x: Math.max(0, x), y: Math.max(0, y) };
  }

  function closeContextMenu() {
    ctxMenu = null;
  }

  function ctxMoveToOther() {
    if (!ctxMenu) return;
    const otherPanel: 'primary' | 'secondary' = panelId === 'primary' ? 'secondary' : 'primary';
    workspace.moveTab(ctxMenu.tabId, otherPanel);
    closeContextMenu();
  }

  function ctxClose() {
    if (!ctxMenu) return;
    onTabClose(ctxMenu.tabId);
    closeContextMenu();
  }

  function handleTabClick(tabId: string) {
    onTabSelect(tabId);
  }

  function handleTabClose(e: MouseEvent | KeyboardEvent, tabId: string) {
    e.stopPropagation();
    onTabClose(tabId);
  }

  function handleMiddleClick(e: MouseEvent, tabId: string) {
    if (e.button === 1) {
      e.preventDefault();
      onTabClose(tabId);
    }
  }

  function handleNewTabChoice(type: WindowType) {
    workspace.closeNewTabPicker();
    onNewTab(panelId, type);
  }

  function toggleNewTabPicker() {
    console.debug('[panel] + clicked', { panelId, wasOpenHere: pickerOpenHere, tabsLen: tabs.length });
    if (pickerOpenHere) {
      workspace.closeNewTabPicker();
    } else {
      workspace.openNewTabPicker(panelId);
    }
  }

  // Drag-source mime tags. `text/elai-tab` = existing tab being moved between
  // panels (move semantics). `application/x-elai-tab-type` = sidebar icon being
  // dropped to OPEN a new tab of that type (open semantics — see Sidebar.svelte).
  const MIME_TAB = 'text/elai-tab';
  const MIME_TAB_TYPE = 'application/x-elai-tab-type';

  function isTabTypeDrag(e: DragEvent): boolean {
    return !!e.dataTransfer && Array.from(e.dataTransfer.types).includes(MIME_TAB_TYPE);
  }

  function handleDragStart(e: DragEvent, tabId: string) {
    if (!e.dataTransfer) return;
    e.dataTransfer.setData(
      MIME_TAB,
      JSON.stringify({ tabId, fromPanel: panelId })
    );
    e.dataTransfer.effectAllowed = 'move';
    workspace.setTabDragging(true);
  }

  function handleDragEnd() {
    // Always clear — fires even on cancelled / failed drops.
    workspace.setTabDragging(false);
    dragOverTabId = null;
    bodyDragHover = false;
  }

  function handleDragOver(e: DragEvent, tabId: string) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = isTabTypeDrag(e) ? 'copy' : 'move';
    }
    dragOverTabId = tabId;
  }

  function handleDragLeave() {
    dragOverTabId = null;
  }

  function handleDrop(e: DragEvent, _targetTabId: string) {
    e.preventDefault();
    dragOverTabId = null;
    bodyDragHover = false;
    if (!e.dataTransfer) return;

    // Sidebar-icon drop = OPEN a new tab of the given type in this panel.
    const type = e.dataTransfer.getData(MIME_TAB_TYPE);
    if (type && (WINDOW_TYPES as string[]).includes(type)) {
      workspace.openTab(panelId, type as WindowType);
      return;
    }

    // Existing tab pill drop = MOVE between panels.
    const raw = e.dataTransfer.getData(MIME_TAB);
    if (!raw) return;
    try {
      const data = JSON.parse(raw) as { tabId: string; fromPanel: string };
      if (data.fromPanel !== panelId) {
        workspace.moveTab(data.tabId, panelId);
      }
    } catch {
      // malformed drag data; ignore
    }
  }

  function handleTabBarDrop(e: DragEvent) {
    handleDrop(e, '');
  }

  // Body drop: only accepts sidebar-icon drags (open semantics). We deliberately
  // ignore tab-pill moves on the body — those are handled by the tab-bar — to keep
  // reorder vs reparent semantics distinct.
  function handleBodyDragOver(e: DragEvent) {
    if (!isTabTypeDrag(e)) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
    bodyDragHover = true;
  }
  function handleBodyDragLeave() {
    bodyDragHover = false;
  }
  function handleBodyDrop(e: DragEvent) {
    if (!isTabTypeDrag(e)) return;
    e.preventDefault();
    bodyDragHover = false;
    const type = e.dataTransfer?.getData(MIME_TAB_TYPE) ?? '';
    if (type && (WINDOW_TYPES as string[]).includes(type)) {
      workspace.openTab(panelId, type as WindowType);
    }
  }

  function closeNewTabMenu(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (pickerOpenHere && !target.closest('.new-tab-menu') && !target.closest('.new-tab-btn')) {
      workspace.closeNewTabPicker();
    }
    if (ctxMenu && !target.closest('.tab-ctx-menu')) {
      closeContextMenu();
    }
  }

  function handleNewTabKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (pickerOpenHere) workspace.closeNewTabPicker();
      if (ctxMenu) closeContextMenu();
    }
  }

  let activeTab = $derived(tabs.find((t) => t.id === activeTabId) ?? tabs[0]);

  // Debug: log every render-state change for the picker so the user can confirm
  // in the Tauri devtools console that the click reached the handler AND that
  // the picker mounted for the correct panel.
  $effect(() => {
    if (pickerOpenHere) {
      console.debug('[panel] picker rendering', { panelId, target: workspace.newTabPickerTarget });
    }
  });
</script>

<svelte:window onclick={closeNewTabMenu} onkeydown={handleNewTabKeydown} />

<div class="panel" data-panel-id={panelId}>
  <div
    class="tab-bar"
    role="tablist"
    tabindex={-1}
    aria-label="{panelId} panel tabs"
    ondragover={(e) => e.preventDefault()}
    ondrop={handleTabBarDrop}
  >
    <div class="tabs-scroll">
      {#each tabs as tab (tab.id)}
        <button
          class="tab"
          class:tab--active={tab.id === activeTabId}
          class:tab--drag-over={dragOverTabId === tab.id}
          role="tab"
          aria-selected={tab.id === activeTabId}
          aria-controls="panel-content-{tab.id}"
          id="tab-{tab.id}"
          draggable={true}
          onclick={() => handleTabClick(tab.id)}
          onmousedown={(e) => handleMiddleClick(e, tab.id)}
          oncontextmenu={(e) => openContextMenu(e, tab.id)}
          ondragstart={(e) => handleDragStart(e, tab.id)}
          ondragend={handleDragEnd}
          ondragover={(e) => handleDragOver(e, tab.id)}
          ondragleave={handleDragLeave}
          ondrop={(e) => handleDrop(e, tab.id)}
        >
          <span class="tab-title">{tab.title}</span>
          <span
            class="tab-close"
            role="button"
            tabindex={-1}
            aria-label="Close {tab.title}"
            onclick={(e) => handleTabClose(e, tab.id)}
            onkeydown={(e) => e.key === 'Enter' && handleTabClose(e, tab.id)}
          >
            <X size={10} stroke-width={2} />
          </span>
        </button>
      {/each}
    </div>

    <div class="tab-bar-end">
      <button
        class="new-tab-btn"
        class:new-tab-btn--prominent={tabs.length === 0}
        onclick={(e) => { e.stopPropagation(); toggleNewTabPicker(); }}
        aria-label="Open new tab"
        aria-expanded={pickerOpenHere}
        aria-haspopup="menu"
        title="New tab"
      >
        <Plus size={13} stroke-width={1.5} />
      </button>

      {#if pickerOpenHere}
        <div class="new-tab-menu" role="menu" aria-label="Choose window type">
          {#each WINDOW_TYPES as type (type)}
            <button
              class="new-tab-option"
              role="menuitem"
              onclick={() => handleNewTabChoice(type)}
            >
              {WINDOW_LABELS[type]}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="panel-content"
    class:panel-content--drag-hover={bodyDragHover}
    id="panel-content-{activeTab?.id ?? 'empty'}"
    role="tabpanel"
    tabindex={0}
    aria-labelledby={activeTab ? `tab-${activeTab.id}` : undefined}
    ondragover={handleBodyDragOver}
    ondragleave={handleBodyDragLeave}
    ondrop={handleBodyDrop}
  >
    {#if activeTab}
      {#key activeTabId}
        <div class="window-host anim-enter-fade">
          <WindowHost type={activeTab.type} />
        </div>
      {/key}
    {:else}
      <div class="panel-empty">
        <p class="panel-empty-msg">No tabs open</p>
        <button
          class="panel-empty-cta"
          onclick={(e) => { e.stopPropagation(); toggleNewTabPicker(); }}
          aria-label="Open new tab picker"
        >
          <Plus size={14} stroke-width={1.5} />
          <span>Open a window</span>
        </button>
        <p class="panel-empty-hint">or drag a sidebar icon here</p>
      </div>
    {/if}
  </div>
</div>

{#if ctxMenu}
  <ul
    class="tab-ctx-menu"
    role="menu"
    aria-label="Tab actions"
    style="left: {ctxMenu.x}px; top: {ctxMenu.y}px;"
  >
    <li>
      <button class="tab-ctx-item" role="menuitem" onclick={ctxMoveToOther}>
        Move to other panel
      </button>
    </li>
    <li>
      <button class="tab-ctx-item" role="menuitem" onclick={ctxClose}>
        Close tab
      </button>
    </li>
  </ul>
{/if}

<style>
  .panel {
    display: flex;
    flex-direction: column;
    flex: 1 1 var(--panel-split, 50%);
    min-width: 320px;
    overflow: hidden;
    background: var(--bg-base);
  }

  .tab-bar {
    display: flex;
    align-items: center;
    height: 32px;
    background: transparent;
    border-bottom: 1px solid transparent;
    flex-shrink: 0;
    /* overflow MUST be visible so the absolutely-positioned .new-tab-menu picker
       can render below the 32px-tall tab-bar without being clipped. Tab pill
       overflow is handled by .tabs-scroll's own overflow-x:auto. */
    overflow: visible;
    position: relative;
    /* Ensure the picker, which sits inside .tab-bar-end, paints above the
       sibling .panel-content. */
    z-index: 2;
    opacity: 0.35;
    transition: opacity 220ms var(--ease-std), background 220ms var(--ease-std), border-color 220ms var(--ease-std);
  }
  .panel:hover .tab-bar,
  .panel:focus-within .tab-bar {
    opacity: 1;
    background: var(--bg-surface);
    border-bottom-color: var(--border-subtle);
  }

  .tabs-scroll {
    display: flex;
    flex-direction: row;
    overflow-x: auto;
    overflow-y: hidden;
    flex: 1 1 auto;
    scrollbar-width: none;
    height: 100%;
  }

  .tabs-scroll::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding: 0 12px;
    background: none;
    border: none;
    border-bottom: 1px solid transparent;
    cursor: pointer;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    white-space: nowrap;
    outline: none;
    position: relative;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std);
    flex-shrink: 0;
    align-self: flex-end;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab:hover .tab-close {
    opacity: 1;
  }

  .tab:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .tab--active {
    color: var(--text-primary);
    border-bottom: 1px solid var(--teal);
  }

  .tab--drag-over {
    background: var(--teal-ghost);
  }

  .tab-title {
    pointer-events: none;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    opacity: 0;
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    transition: opacity 120ms var(--ease-std), color 120ms var(--ease-std), background 120ms var(--ease-std);
    cursor: pointer;
  }

  .tab-close:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .tab-bar-end {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    padding: 0 4px;
    position: relative;
  }

  .new-tab-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: var(--r-xs);
    outline: none;
    opacity: 0;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std), opacity 220ms var(--ease-std);
  }
  .panel:hover .new-tab-btn,
  .panel:focus-within .new-tab-btn { opacity: 1; }

  .new-tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .new-tab-btn:focus-visible {
    box-shadow: var(--ring);
  }

  /* When the panel is empty, the only meaningful action is "open a tab" — keep
     the + button visible without requiring hover. */
  .new-tab-btn--prominent {
    opacity: 1;
    color: var(--teal);
    background: var(--teal-ghost);
  }
  .new-tab-btn--prominent:hover {
    color: var(--teal-bright);
    background: var(--teal-soft);
  }

  .new-tab-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    padding: 4px;
    z-index: 50;
    min-width: 140px;
    animation: enterLift 180ms var(--ease-out) both;
  }

  @keyframes enterLift {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  /* Outlined teal-line border with teal-ghost hover — mirrors CommandWindow slash palette. */
  .new-tab-option {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    margin: 2px 0;
    background: transparent;
    border: 1px solid var(--teal-line);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    border-radius: var(--r-sm);
    outline: none;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std), border-color 120ms var(--ease-std);
  }

  .new-tab-option:hover {
    color: var(--teal-bright);
    background: var(--teal-ghost);
    border-color: var(--teal);
  }

  .new-tab-option:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .panel-content {
    flex: 1 1 auto;
    overflow: hidden;
    background: var(--bg-base);
    position: relative;
    transition: background 140ms var(--ease-std), box-shadow 140ms var(--ease-std);
  }

  .panel-content--drag-hover {
    background: var(--teal-ghost);
    box-shadow: inset 0 0 0 1px var(--teal-line);
  }

  .panel-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .panel-empty-msg {
    margin: 0;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .panel-empty-cta {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    color: var(--teal);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
    outline: none;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std), border-color 120ms var(--ease-std);
  }

  .panel-empty-cta:hover {
    color: var(--teal-bright);
    background: var(--teal-ghost);
    border-color: var(--teal);
  }

  .panel-empty-cta:focus-visible {
    box-shadow: var(--ring);
  }

  .panel-empty-hint {
    margin: 0;
    opacity: 0.6;
    font-size: 10px;
  }

  .window-host {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }

  /* Tab right-click context menu — reuses outlined-button look from new-tab-menu */
  .tab-ctx-menu {
    position: fixed;
    z-index: 200;
    margin: 0;
    padding: 4px;
    list-style: none;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    min-width: 180px;
    animation: enterLift 140ms var(--ease-out) both;
  }

  .tab-ctx-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    border-radius: var(--r-sm);
    outline: none;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std);
  }

  .tab-ctx-item:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .tab-ctx-item:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }
</style>
