<script lang="ts">
  // E-29 Tier-A — SpendDrawer.
  // Surfaces the existing `get_cloud_spend` Tauri command (commands.rs:4335)
  // which rolls up the C-5 CloudSpend SSB rows via SsbViewer::cloud_spend_summary.
  // Wraps the existing pricing surface from providers/src/pricing.rs (C-14).
  // No new backend wiring — Tier-A is pure dashboard read of existing producers.

  import { onMount, onDestroy } from 'svelte';
  import { get_cloud_spend } from '$lib/api/stub';
  import type { CloudSpendInfo } from '$lib/api/types';

  const POLL_INTERVAL_MS = 10_000;

  let info = $state<CloudSpendInfo | null>(null);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      info = await get_cloud_spend();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void refresh();
    timer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
  });

  onDestroy(() => {
    if (timer !== null) clearInterval(timer);
    timer = null;
  });

  function fmtUsd(n: number): string {
    if (n < 0.01) return `$${n.toFixed(4)}`;
    if (n < 1) return `$${n.toFixed(3)}`;
    return `$${n.toFixed(2)}`;
  }

  function fmtTokens(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
    return `${n}`;
  }
</script>

<section class="spend-drawer" aria-label="Cloud spend">
  <header class="hd">
    <h3 class="hd-title">cloud spend</h3>
    <span class="hd-hint">cloud_spend SSB rollup · polled 10s</span>
  </header>

  {#if loading && info === null}
    <div class="state state--loading">loading spend…</div>
  {:else if error}
    <div class="state state--error">spend query failed: {error}</div>
  {:else if info === null}
    <div class="state state--empty">no cloud_spend rows yet</div>
  {:else}
    <div class="grid">
      <div class="cell">
        <div class="cell-key">latest task</div>
        <div class="cell-val">{fmtUsd(info.task_usd)}</div>
      </div>
      <div class="cell">
        <div class="cell-key">today total</div>
        <div class="cell-val cell-val--strong">{fmtUsd(info.day_usd)}</div>
      </div>
      <div class="cell">
        <div class="cell-key">cache creation</div>
        <div class="cell-val">{fmtTokens(info.cache_creation_tokens)} tok</div>
      </div>
      <div class="cell">
        <div class="cell-key">cache read</div>
        <div class="cell-val">{fmtTokens(info.cache_read_tokens)} tok</div>
      </div>
      <div class="cell">
        <div class="cell-key">redactions</div>
        <div class="cell-val">{info.redaction_count}</div>
      </div>
    </div>
    <div class="note">
      cache-read saves ~10x vs cache-creation; ratio is the privacy-redactor x cost-cap signal
    </div>
  {/if}
</section>

<style>
  .spend-drawer {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-3);
    background: var(--bg-surface);
    border-radius: var(--r-md);
    border: 1px solid var(--border-subtle);
  }
  .hd {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
  }
  .hd-title {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .hd-hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--s-2);
  }
  .cell {
    background: var(--bg-base);
    padding: var(--s-2);
    border-radius: var(--r-sm);
    border: 1px solid var(--border-subtle);
  }
  .cell-key {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .cell-val {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-primary);
    margin-top: 2px;
  }
  .cell-val--strong {
    color: var(--signal-blue);
    font-weight: 500;
  }
  .state {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    padding: var(--s-2);
  }
  .state--error { color: var(--fail); }
  .state--empty { color: var(--text-ghost); }
  .note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding-top: var(--s-1);
  }
</style>
