<script lang="ts">
  // E-4 Task 3: PinnedRulesPanel.
  //
  // Renders all user-stated constraints from the session in a single
  // collapsible panel. Sources:
  //   (a) rules.db ACTIVE rows (existing crate::rules surface).
  //   (b) per-session ad-hoc constraints surfaced as
  //       SsbActionType::ChatUxCritiqueEvent rows with
  //       ChatUxCritiqueKind::UserRulePinned.
  //
  // Click-to-remove emits ChatUxCritiqueKind::UserRuleUnpinned via a
  // future Tauri command (kept dark until commands_e4.rs lands; this
  // panel calls the command name today, falls back to a console
  // warning if the command isn't registered).
  //
  // Defeats "lost in the middle" for user instructions (PRIMARY:
  // aclanthology.org/2024.tacl-1.9 — Liu et al. 2024 Table 3,
  // mid-context recall ~30% lower than ends).
  //
  // Svelte 5 runes only. No createEventDispatcher; no $: labels.
  // Wire-dark unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1 on the producer
  // side — this panel still renders, just with empty data.

  import { onMount, onDestroy } from 'svelte';

  interface PinnedRule {
    rule_id: string;
    text: string;
    source: 'rules_db' | 'session_pin';
    /** Unix ms when the row was created. */
    ts_ms: number;
  }

  interface Props {
    pollMs?: number;
    collapsedDefault?: boolean;
  }

  const { pollMs = 15_000, collapsedDefault = true }: Props = $props();

  let rules = $state<PinnedRule[]>([]);
  // Note: collapsedDefault is the INITIAL value only; the state below
  // becomes the source-of-truth after first render. The
  // "state_referenced_locally" lint guidance is acknowledged here —
  // we only capture the initial prop value by design (prop changes
  // after mount do not retroactively recollapse the panel).
  // svelte-ignore state_referenced_locally
  let collapsed = $state<boolean>(collapsedDefault);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail =
      !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.warn(`[pinned-rules] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  async function reload() {
    // commands_e4::get_pinned_rules — wire-dark until the Tauri
    // command lands; falls back to empty list.
    const out = await callInvoke<PinnedRule[]>('get_pinned_rules', {});
    if (out !== undefined) {
      rules = out;
      loadError = null;
    }
  }

  async function unpin(rule_id: string, source: 'rules_db' | 'session_pin') {
    // commands_e4::unpin_user_rule — wire-dark until landing. The call
    // is no-op on missing handler but rules optimistically removes
    // session_pin rows so the UI stays responsive.
    await callInvoke('unpin_user_rule', { rule_id, source });
    rules = rules.filter((r) => r.rule_id !== rule_id);
  }

  onMount(() => {
    void reload();
    pollHandle = setInterval(() => void reload(), pollMs);
  });

  onDestroy(() => {
    if (pollHandle !== null) clearInterval(pollHandle);
  });

  function fmtAgo(tsMs: number): string {
    if (!Number.isFinite(tsMs) || tsMs <= 0) return '—';
    const deltaMs = Date.now() - tsMs;
    if (deltaMs < 0) return '—';
    const sec = Math.floor(deltaMs / 1000);
    if (sec < 60) return `${sec}s`;
    const min = Math.floor(sec / 60);
    if (min < 60) return `${min}m`;
    const hr = Math.floor(min / 60);
    if (hr < 24) return `${hr}h`;
    return `${Math.floor(hr / 24)}d`;
  }

  let count = $derived(rules.length);
</script>

<section
  class="prp"
  class:prp--collapsed={collapsed}
  aria-label="pinned rules panel"
>
  <button
    type="button"
    class="prp-header"
    onclick={() => (collapsed = !collapsed)}
    aria-expanded={!collapsed}
  >
    <span class="prp-glyph" aria-hidden="true">{collapsed ? '▸' : '▾'}</span>
    <span class="prp-title">pinned rules</span>
    <span class="prp-count mono">{count}</span>
  </button>
  {#if !collapsed}
    <div class="prp-body">
      {#if loadError}
        <div class="prp-error mono">{loadError}</div>
      {/if}
      {#if rules.length === 0}
        <div class="prp-empty">no pinned rules</div>
      {:else}
        <ul class="prp-list">
          {#each rules as r (r.rule_id)}
            <li class="prp-row">
              <span class="prp-source mono prp-source--{r.source}">
                {r.source === 'rules_db' ? 'db' : 'pin'}
              </span>
              <span class="prp-text">{r.text}</span>
              <span class="prp-age mono">{fmtAgo(r.ts_ms)}</span>
              <button
                type="button"
                class="prp-remove"
                title="unpin"
                aria-label="unpin rule {r.rule_id}"
                onclick={() => void unpin(r.rule_id, r.source)}
              >×</button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</section>

<style>
  .prp {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-md, 4px);
    background: var(--bg-surface);
    font-family: var(--font-sans);
    font-size: 12px;
    overflow: hidden;
  }
  .prp-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    width: 100%;
    text-align: left;
  }
  .prp-header:hover {
    background: var(--bg-surface-hover, rgba(255, 255, 255, 0.03));
  }
  .prp-glyph {
    color: var(--text-ghost);
    font-size: 10px;
  }
  .prp-title {
    flex: 1;
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .prp-count {
    color: var(--text-ghost);
    font-size: 11px;
  }
  .prp-body {
    border-top: 1px solid var(--border-subtle);
    padding: 6px 8px;
  }
  .prp-error {
    color: var(--red, #f87171);
    font-size: 10px;
    margin-bottom: 4px;
  }
  .prp-empty {
    color: var(--text-ghost);
    font-style: italic;
    padding: 4px 0;
  }
  .prp-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .prp-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 0;
  }
  .prp-source {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    width: 28px;
    flex-shrink: 0;
  }
  .prp-source--rules_db {
    color: var(--blue, #60a5fa);
  }
  .prp-source--session_pin {
    color: var(--teal, #2dd4bf);
  }
  .prp-text {
    flex: 1;
    color: var(--text-primary);
  }
  .prp-age {
    color: var(--text-ghost);
    font-size: 10px;
  }
  .prp-remove {
    background: transparent;
    border: none;
    color: var(--text-ghost);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 4px;
  }
  .prp-remove:hover {
    color: var(--red, #f87171);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
