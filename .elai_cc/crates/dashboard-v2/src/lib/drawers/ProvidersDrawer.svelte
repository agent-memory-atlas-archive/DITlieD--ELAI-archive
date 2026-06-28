<script lang="ts">
  import { onMount } from 'svelte';
  import { Badge, Button, Scrollbar, UnwiredNote } from '$lib/ui';
  import { providersData } from '$lib/state/drawers.svelte';
  import type { VramGauge } from '$lib/api/types';

  onMount(() => {
    providersData.bootstrap();
    return providersData.teardown;
  });

  const NOW = Date.now();

  // ------------------------------------------------------------------
  // Section 1 — MODEL POOL — derived from live sidecar+vram polling.
  // ------------------------------------------------------------------
  const sidecar = $derived(providersData.sidecarStatus);
  const vramGauges = $derived<VramGauge[]>(providersData.vramGauges);
  const mcpStatus = $derived(providersData.mcpStatus);
  const inferenceMetrics = $derived(providersData.inferenceMetrics);

  interface ModelPoolEntry {
    model: string;
    gpu_id: number;
    used_mb: number;
    last_used_ts: number;
    active: boolean;
  }

  const modelPool = $derived<ModelPoolEntry[]>(vramGauges.map(g => ({
    model: sidecar.model,
    gpu_id: g.gpu_id,
    used_mb: g.used_mb,
    last_used_ts: NOW,
    active: sidecar.loaded,
  })));

  function formatMb(mb: number): string {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${mb} MB`;
  }

  function relTime(ts: number): string {
    const diff = NOW - ts;
    const m = Math.floor(diff / 60_000);
    if (m === 0) return 'just now';
    if (m < 60) return `${m}m ago`;
    const h = Math.floor(m / 60);
    return `${h}h ago`;
  }

  // ------------------------------------------------------------------
  // Section 2 — VRAM PER GPU
  // ------------------------------------------------------------------
  function vramBarColor(pct: number): string {
    if (pct > 95) return 'var(--fail)';
    if (pct > 70) return 'var(--warn)';
    return 'var(--teal)';
  }

  // ------------------------------------------------------------------
  // Section 3 — SIDECAR STATUS
  // ------------------------------------------------------------------
  interface FlagChip {
    label: string;
    active: boolean;
  }

  const flags = $derived<FlagChip[]>([
    { label: 'FA2',        active: sidecar.fa2 },
    { label: 'DoLa',       active: sidecar.dola },
    { label: 'Steering',   active: sidecar.steering },
    { label: 'TurboQuant', active: sidecar.turboquant !== 'OFF' },
  ]);

  // ------------------------------------------------------------------
  // Section 4 — TOKENS/SEC LAST 60s (mini-sparkline)
  // 10 hardcoded data points from mock-derived range
  // ------------------------------------------------------------------
  const sparkPoints = [32, 35, 38, 36, 40, 38, 42, 39, 38, 38]; // tok/s
  const sparkMin = Math.min(...sparkPoints);
  const sparkMax = Math.max(...sparkPoints);
  const sparkRange = sparkMax - sparkMin || 1;

  const SVG_W = 150;
  const SVG_H = 24;
  const n = sparkPoints.length;

  function sparkX(i: number): number {
    return (i / (n - 1)) * SVG_W;
  }

  function sparkY(v: number): number {
    return SVG_H - ((v - sparkMin) / sparkRange) * (SVG_H - 4) - 2;
  }

  const sparkPath = sparkPoints
    .map((v, i) => `${i === 0 ? 'M' : 'L'}${sparkX(i).toFixed(1)},${sparkY(v).toFixed(1)}`)
    .join(' ');

  // ------------------------------------------------------------------
  // Section 5 — MCP BRIDGE
  // ------------------------------------------------------------------
  let mcpReconnecting = $state(false);

  function handleMcpReconnect() {
    mcpReconnecting = true;
    console.info('[stub] mcp.reconnect', { url: mcpStatus.bridge_url });
    setTimeout(() => {
      mcpReconnecting = false;
    }, 1500);
  }
</script>

<div class="providers-drawer">
  <Scrollbar>
    <div class="drawer-body">

      <!-- ── MODEL POOL ──────────────────────────────────────── -->
      <section class="section">
        <div class="section-heading">MODEL POOL</div>
        {#if providersData.unwired.modelPool}
          <UnwiredNote
            phase="3J"
            feature="loaded models per GPU + last-used timestamps + idle-eviction events"
            note="ModelPool not exposed via Tauri command; current row is synthesised from sidecar+vram. Pending dedicated get_model_pool query."
          />
        {:else if modelPool.length === 0}
          <div class="empty-note">No models loaded.</div>
        {:else}
          {#each modelPool as entry}
            <div class="model-row">
              <span class="model-name" class:model-name--active={entry.active}>
                {entry.model}
              </span>
              <Badge tone={entry.active ? 'teal' : 'ghost'} size="xs">
                cuda:{entry.gpu_id}
              </Badge>
              <span class="model-meta">{formatMb(entry.used_mb)}</span>
              <span class="model-meta">{relTime(entry.last_used_ts)}</span>
            </div>
          {/each}
        {/if}
      </section>

      <!-- ── VRAM PER GPU ────────────────────────────────────── -->
      <section class="section">
        <div class="section-heading">VRAM PER GPU</div>
        {#each vramGauges as g}
          <div class="gpu-vram-block">
            <div class="gpu-header">
              <span class="gpu-name">{g.gpu_name}</span>
              <span
                class="gpu-pct"
                style="color:{vramBarColor(g.pct_used)};"
              >
                {g.pct_used}%
              </span>
            </div>
            <div class="gpu-meta">
              {formatMb(g.used_mb)} used &middot; {formatMb(g.free_mb)} free &middot; {formatMb(g.total_mb)} total
            </div>
            <div class="vram-bar-wrap">
              <div
                class="vram-bar"
                style="width:{g.pct_used}%; background:{vramBarColor(g.pct_used)};"
              ></div>
            </div>
          </div>
        {/each}
      </section>

      <!-- ── SIDECAR STATUS ──────────────────────────────────── -->
      <section class="section">
        <div class="section-heading">SIDECAR STATUS</div>
        <div class="sidecar-card">
          <div class="sidecar-model">{sidecar.model}</div>
          <div class="sidecar-flags">
            {#each flags as f}
              <span
                class="flag-chip"
                class:flag-chip--on={f.active}
              >
                {f.label}
              </span>
            {/each}
          </div>
          <div class="sidecar-perf">
            {sidecar.tokens_per_sec} tok/s &middot; p50 {sidecar.latency_p50_ms}ms &middot; p99 {sidecar.latency_p99_ms}ms
          </div>
        </div>
      </section>

      <!-- ── TOKENS/SEC LAST 60s ────────────────────────────── -->
      <section class="section">
        <div class="section-heading">TOKENS/SEC LAST 60s</div>
        {#if providersData.unwired.tokensPerSec}
          <UnwiredNote
            phase="3J / B-4"
            feature="tok/s sparkline over last 60 s + p50/p99 latency"
            note="get_inference_metrics returns zeros; per-second history not aggregated by SSB yet"
          />
        {:else}
          <div class="sparkline-wrap">
            <svg
              class="sparkline"
              width={SVG_W}
              height={SVG_H}
              viewBox="0 0 {SVG_W} {SVG_H}"
              aria-label="tokens per second sparkline"
            >
              <path
                d={sparkPath}
                fill="none"
                stroke="var(--teal)"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <span class="sparkline-label">{inferenceMetrics.tokens_per_sec_last_60s} tok/s avg</span>
          </div>
        {/if}
      </section>

      <!-- ── MCP BRIDGE ─────────────────────────────────────── -->
      <section class="section">
        <div class="section-heading">MCP BRIDGE</div>
        <div class="mcp-row">
          <span
            class="mcp-status-dot"
            class:mcp-status-dot--connected={mcpStatus.connected}
          ></span>
          <span class="mcp-status-label">
            {mcpStatus.connected ? 'CONNECTED' : 'DISCONNECTED'}
          </span>
          {#if mcpStatus.ping_ms !== undefined}
            <span class="mcp-ping">{mcpStatus.ping_ms}ms</span>
          {/if}
          <button
            class="mcp-reconnect"
            onclick={handleMcpReconnect}
            disabled={mcpReconnecting}
            type="button"
          >
            {mcpReconnecting ? 'connecting...' : '[Reconnect]'}
          </button>
        </div>
        <div class="mcp-url">{mcpStatus.bridge_url}</div>
      </section>

      <!-- ── GPU MULTIPLEXING EVENTS ────────────────────────── -->
      <section class="section section--last">
        <div class="section-heading">GPU MULTIPLEXING EVENTS</div>
        <UnwiredNote
          phase="3J"
          feature="colocation / model-swap / OOM-guard events per GPU"
          note="GPU multiplexing events not emitted to SSB; producer in elai_providers ModelPool not yet wired"
        />
      </section>

    </div>
  </Scrollbar>
</div>

<style>
  .providers-drawer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .drawer-body {
    padding: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
  }

  .section-heading {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-secondary);
    text-transform: uppercase;
    margin-bottom: var(--s-3);
  }

  /* ── MODEL POOL ───────────────────────────────── */
  .model-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px var(--s-2);
    border-radius: var(--r-xs);
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .model-row:hover {
    background: var(--bg-hover);
  }

  .model-name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-name--active {
    color: var(--teal);
  }

  .model-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  /* ── VRAM ─────────────────────────────────────── */
  .gpu-vram-block {
    margin-bottom: var(--s-3);
  }

  .gpu-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 3px;
  }

  .gpu-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 200px;
  }

  .gpu-pct {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    flex-shrink: 0;
  }

  .gpu-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin-bottom: 4px;
  }

  .vram-bar-wrap {
    height: 6px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    overflow: hidden;
  }

  .vram-bar {
    height: 100%;
    border-radius: var(--r-xs);
    transition: width var(--dur-base) var(--ease-out);
  }

  /* ── SIDECAR ──────────────────────────────────── */
  .sidecar-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-3);
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .sidecar-model {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sidecar-flags {
    display: flex;
    gap: var(--s-1);
    flex-wrap: wrap;
  }

  .flag-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: var(--r-pill);
    border: 1px solid var(--border-subtle);
    color: var(--text-ghost);
    background: transparent;
    transition: color var(--dur-fast) var(--ease-std), background-color var(--dur-fast) var(--ease-std);
  }

  .flag-chip--on {
    color: var(--teal);
    background: var(--teal-ghost);
    border-color: var(--teal-line);
  }

  .sidecar-perf {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── SPARKLINE ────────────────────────────────── */
  .sparkline-wrap {
    display: flex;
    align-items: center;
    gap: var(--s-3);
  }

  .sparkline {
    display: block;
    flex-shrink: 0;
  }

  .sparkline-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── MCP BRIDGE ───────────────────────────────── */
  .mcp-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-1);
  }

  .mcp-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-ghost);
    flex-shrink: 0;
    transition: background-color var(--dur-fast) var(--ease-std);
  }

  .mcp-status-dot--connected {
    background: var(--teal);
    animation: breath 2.2s ease-in-out infinite;
  }

  @keyframes breath {
    0%   { opacity: 0.2; }
    50%  { opacity: 0.55; }
    100% { opacity: 0.2; }
  }

  .mcp-status-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.06em;
  }

  .mcp-ping {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .mcp-reconnect {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--r-xs);
    transition: color var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .mcp-reconnect:hover:not(:disabled) {
    color: var(--teal);
  }

  .mcp-reconnect:focus-visible {
    box-shadow: var(--ring);
  }

  .mcp-reconnect:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .mcp-url {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ── COLOCATION EVENTS ────────────────────────── */
  .colocate-row {
    display: flex;
    gap: var(--s-3);
    padding: 4px 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .colocate-row:last-child {
    border-bottom: none;
  }

  .colocate-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
    width: 52px;
  }

  .colocate-desc {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    line-height: var(--leading-code);
  }

  .empty-note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding: var(--s-2) 0;
  }

  .section--last {
    padding-bottom: var(--s-6);
  }
</style>
