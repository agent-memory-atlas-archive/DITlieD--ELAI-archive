<script lang="ts">
  import {
    Database,
    Brain,
    Network,
    ShieldCheck,
    Telescope,
    Cpu,
    FolderSearch,
    CalendarClock,
    Plug,
    Globe,
    Sparkles,
    BookUser,
    NotebookText,
    Regex,
    ShieldAlert,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import type { LucideProps } from '@lucide/svelte';
  import type { DrawerKey } from '../api/types';
  import { workspace } from '../state/workspace.svelte';

  // @lucide/svelte v1 exports Svelte 5 function components.
  type IconCtor = Component<LucideProps>;

  // Uppercase keys used for display; stored as lowercase DrawerKey in workspace
  type DrawerKeyUpper =
    | 'KNOWLEDGE'
    | 'REASONING'
    | 'ORCHESTRATION'
    | 'RULES'
    | 'RESEARCH'
    | 'PROVIDERS'
    | 'PROJECT-INTELLIGENCE'
    | 'ANTI-STALENESS'
    | 'TOOLS-HOOKS'
    | 'WEB-QUALITY'
    | 'COSMOS'
    | 'MEMORY-FACTS'
    | 'CHAT-MEMORY'
    | 'GRAMMAR-COMPILE'
    // F-1 / A5: VibeGate per-surface drawer.
    | 'VIBEGATE'
    // F-2a / A5: SecurityRules per-surface drawer.
    | 'SECURITY-RULES'
    // F-9 / Slice D5: DepsSecurityPanel per-surface drawer.
    | 'DEPS-SECURITY'
    // F-10 / Slice A5: SecurityMonotonicityDrawer per-surface drawer.
    | 'SECURITY-MONOTONICITY'
    // F-5 / Slice A5: StalenessQuarantineDrawer per-surface drawer.
    | 'STALENESS-QUARANTINE'
    // F-7 / Slice A5: TemplateRankingDrawer per-surface drawer.
    | 'TEMPLATE-RANKING'
    // F-13 / Slice A5: SessionArchiveDrawer per-surface drawer.
    | 'SESSION-ARCHIVE';

  interface DrawerItem {
    key: DrawerKeyUpper;
    lowerKey: DrawerKey;
    label: string;
    Icon: IconCtor;
  }

  const drawerItems: DrawerItem[] = [
    { key: 'KNOWLEDGE',            lowerKey: 'knowledge',            label: 'KNOWLEDGE',            Icon: Database },
    { key: 'REASONING',            lowerKey: 'reasoning',            label: 'REASONING',            Icon: Brain },
    { key: 'ORCHESTRATION',        lowerKey: 'orchestration',        label: 'ORCHESTRATION',        Icon: Network },
    { key: 'RULES',                lowerKey: 'rules',                label: 'RULES',                Icon: ShieldCheck },
    { key: 'RESEARCH',             lowerKey: 'research',             label: 'RESEARCH',             Icon: Telescope },
    { key: 'PROVIDERS',            lowerKey: 'providers',            label: 'PROVIDERS',            Icon: Cpu },
    { key: 'PROJECT-INTELLIGENCE', lowerKey: 'project-intelligence', label: 'PROJECT INTELLIGENCE', Icon: FolderSearch },
    { key: 'ANTI-STALENESS',       lowerKey: 'anti-staleness',       label: 'ANTI-STALENESS',       Icon: CalendarClock },
    { key: 'TOOLS-HOOKS',          lowerKey: 'tools-hooks',          label: 'TOOLS & HOOKS',        Icon: Plug },
    { key: 'WEB-QUALITY',          lowerKey: 'web-quality',          label: 'WEB QUALITY',          Icon: Globe },
    // B-42 — Cosmos artifact tier (read-only status; refresh deferred).
    { key: 'COSMOS',               lowerKey: 'cosmos',               label: 'COSMOS',               Icon: Sparkles },
    // E-5 — User modeling + long-term memory facts.
    { key: 'MEMORY-FACTS',         lowerKey: 'memory-facts',         label: 'MEMORY FACTS',         Icon: BookUser },
    // E-1 dashboard slice — chat-memory (MemoryNotes + CoreBlock).
    { key: 'CHAT-MEMORY',          lowerKey: 'chat-memory',          label: 'CHAT MEMORY',          Icon: NotebookText },
    // E-35 — XGrammar grammar-cache visibility.
    { key: 'GRAMMAR-COMPILE',      lowerKey: 'grammar-compile',      label: 'GRAMMAR COMPILE',      Icon: Regex },
    // F-1 / A5: VibeGate per-surface drawer. Wire-dark unless ELAI_VIBE_GATE_ENABLE=1.
    { key: 'VIBEGATE',             lowerKey: 'vibegate',             label: 'VIBE GATE',            Icon: ShieldAlert },
    // F-2a / A5: SecurityRules per-surface drawer. Wire-dark unless ELAI_SECURITY_RULE_BLOCKED_ENABLE=1.
    { key: 'SECURITY-RULES',       lowerKey: 'security-rules',       label: 'SECURITY RULES',       Icon: ShieldAlert },
    // F-9 / Slice D5: DepsSecurityPanel per-surface drawer. Default-ON (B1/W-2).
    { key: 'DEPS-SECURITY',        lowerKey: 'deps-security',        label: 'DEPS SECURITY',        Icon: ShieldAlert },
    // F-10 / Slice A5: SecurityMonotonicityDrawer per-surface drawer. Default-ON (W-2).
    { key: 'SECURITY-MONOTONICITY', lowerKey: 'security-monotonicity', label: 'SECURITY MONOTONICITY', Icon: ShieldAlert },
    // F-5 / Slice A5: StalenessQuarantineDrawer per-surface drawer. No ENABLE gate (W-2).
    { key: 'STALENESS-QUARANTINE', lowerKey: 'staleness-quarantine', label: 'STALENESS QUARANTINE', Icon: CalendarClock },
    // F-7 / Slice A5: TemplateRankingDrawer per-surface drawer. No ENABLE gate (W-2).
    { key: 'TEMPLATE-RANKING',     lowerKey: 'template-ranking',     label: 'TEMPLATE RANKING',     Icon: Sparkles },
    // F-13 / Slice A5: SessionArchiveDrawer per-surface drawer. Producer gate
    // ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE is default-OFF / wire-dark (forensic opt-in).
    { key: 'SESSION-ARCHIVE',      lowerKey: 'session-archive',      label: 'SESSION ARCHIVE',      Icon: Database },
  ];

  function tooltip(node: HTMLElement, opts: { label: string }) {
    let tip: HTMLElement | null = null;
    let showTimer: ReturnType<typeof setTimeout> | null = null;

    function show() {
      if (!tip) {
        tip = document.getElementById('rail-tooltip') as HTMLElement | null;
        if (!tip) {
          tip = document.createElement('div');
          tip.id = 'rail-tooltip';
          tip.className = 'rail-tooltip-global';
          document.body.appendChild(tip);
        }
      }
      const rect = node.getBoundingClientRect();
      tip.textContent = opts.label;
      tip.style.top = `${rect.top + rect.height / 2}px`;
      tip.style.right = `${window.innerWidth - rect.left + 8}px`;
      tip.style.left = 'auto';
      tip.style.opacity = '1';
    }

    function hide() {
      if (tip) tip.style.opacity = '0';
    }

    function onMouseEnter() {
      showTimer = setTimeout(show, 400);
    }

    function onMouseLeave() {
      if (showTimer) { clearTimeout(showTimer); showTimer = null; }
      hide();
    }

    node.addEventListener('mouseenter', onMouseEnter);
    node.addEventListener('mouseleave', onMouseLeave);

    return {
      update(newOpts: { label: string }) { opts = newOpts; },
      destroy() {
        node.removeEventListener('mouseenter', onMouseEnter);
        node.removeEventListener('mouseleave', onMouseLeave);
        if (showTimer) clearTimeout(showTimer);
      },
    };
  }

  function toggleDrawer(lowerKey: DrawerKey) {
    workspace.toggleDrawer(lowerKey);
  }
</script>

<nav class="rail" aria-label="Drawer navigation">
  {#each drawerItems as item (item.key)}
    {@const isActive = workspace.activeDrawer === item.lowerKey}
    <button
      class="rail-btn"
      class:rail-btn--active={isActive}
      onclick={() => toggleDrawer(item.lowerKey)}
      aria-label="{item.label} drawer"
      aria-expanded={isActive}
      use:tooltip={{ label: item.label }}
    >
      <item.Icon size={16} stroke-width={1.5} />
    </button>
  {/each}
</nav>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    width: 28px;
    height: 100%;
    background: var(--bg-void);
    align-items: center;
    padding: 8px 0;
    gap: 2px;
  }

  .rail-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-ghost);
    outline: none;
    border-radius: var(--r-xs);
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std);
    flex-shrink: 0;
  }

  .rail-btn:hover {
    color: var(--text-secondary);
  }

  .rail-btn:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .rail-btn--active {
    color: var(--teal);
    background: transparent;
  }
</style>

<svelte:head>
  <style>
    .rail-tooltip-global {
      position: fixed;
      transform: translateY(-50%);
      background: var(--bg-surface);
      border: 1px solid var(--border-default);
      border-radius: 4px;
      padding: 4px 8px;
      font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, monospace;
      font-size: 10px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--text-primary);
      white-space: nowrap;
      pointer-events: none;
      z-index: 9999;
      opacity: 0;
      transition: opacity 100ms ease;
    }
  </style>
</svelte:head>
