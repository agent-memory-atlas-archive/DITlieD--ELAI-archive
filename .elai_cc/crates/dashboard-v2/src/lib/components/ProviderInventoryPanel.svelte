<!--
  J-30 ProviderInventoryPanel — provider inventory + BYOK configure,
  split out of RuntimePanel to clear the MR-1 1000-line BLOCK.

  Svelte 5 runes only ($state / $derived / $effect). No props; the
  panel owns its own 10s poll against list_provider_inventory and
  the descriptor-driven per-row Test/Save form.
-->
<script lang="ts">
  import {
    listProviderInventory,
    providerConfigDescriptor,
    configureProvider,
  } from '$lib/api/stub';
  import type {
    InventoryStatusJson,
    BackendConfigDescriptorJson,
    ConfigureProviderReport,
  } from '$lib/api/types';

  let inventoryRows = $state<InventoryStatusJson[]>([]);
  let inventoryError = $state<string | null>(null);
  let inventoryPending = $state(false);

  let byokBackend = $state<string>('');
  let byokDescriptor = $state<BackendConfigDescriptorJson | null>(null);
  let byokApiKey = $state('');
  let byokBaseUrl = $state('');
  let byokModel = $state('');
  let byokBusy = $state(false);
  let byokReport = $state<ConfigureProviderReport | null>(null);

  async function refreshInventory(): Promise<void> {
    inventoryPending = true;
    try {
      inventoryRows = await listProviderInventory();
      inventoryError = null;
    } catch (err) {
      inventoryError = err instanceof Error ? err.message : String(err);
    } finally {
      inventoryPending = false;
    }
  }

  $effect(() => {
    void refreshInventory();
    const id = setInterval(() => {
      void refreshInventory();
    }, 10_000);
    return () => clearInterval(id);
  });

  function formatVerifiedAt(ms: number | null | undefined): string {
    if (ms == null) return 'never';
    return new Date(ms).toLocaleString();
  }

  async function onSelectInventoryBackend(backend: string): Promise<void> {
    byokBackend = backend;
    byokApiKey = '';
    byokModel = '';
    byokReport = null;
    try {
      const d = await providerConfigDescriptor(backend);
      byokDescriptor = d;
      byokBaseUrl = d.default_base_url ?? '';
    } catch (err) {
      byokDescriptor = null;
      byokBaseUrl = '';
      byokReport = {
        backend,
        ok: false,
        health: 'unavailable',
        error: err instanceof Error ? err.message : String(err),
        verified_at: null,
        saved: false,
      };
    }
  }

  async function onByokTest(): Promise<void> {
    if (!byokBackend || byokBusy) return;
    byokBusy = true;
    try {
      byokReport = await configureProvider({
        backend: byokBackend,
        api_key: byokApiKey || null,
        base_url: byokBaseUrl || null,
        model: byokModel || null,
        save: false,
      });
    } catch (err) {
      byokReport = {
        backend: byokBackend,
        ok: false,
        health: 'unavailable',
        error: err instanceof Error ? err.message : String(err),
        verified_at: null,
        saved: false,
      };
    } finally {
      byokBusy = false;
    }
  }

  async function onByokSave(): Promise<void> {
    if (!byokBackend || byokBusy) return;
    byokBusy = true;
    try {
      byokReport = await configureProvider({
        backend: byokBackend,
        api_key: byokApiKey || null,
        base_url: byokBaseUrl || null,
        model: byokModel || null,
        save: true,
      });
      if (byokReport && byokReport.ok && byokReport.saved) {
        await refreshInventory();
      }
    } catch (err) {
      byokReport = {
        backend: byokBackend,
        ok: false,
        health: 'unavailable',
        error: err instanceof Error ? err.message : String(err),
        verified_at: null,
        saved: false,
      };
    } finally {
      byokBusy = false;
    }
  }

  const byokSecretKey = $derived(
    byokDescriptor?.keys.find((k) => k.secret) ?? null,
  );
  const byokShowBaseUrl = $derived(byokDescriptor?.supports_base_url ?? false);
</script>

<section class="provider-inventory-panel" aria-label="Provider inventory and BYOK">
  <h4>Provider inventory &amp; BYOK</h4>
  <p class="provider-inventory-panel__hint">
    Per-backend configuration status and verified-at timestamp.
    Select a row to load the descriptor-driven BYOK form, then
    Test (no save) or Save (stores key + stamps verified_at).
  </p>
  {#if inventoryError}
    <p class="provider-inventory-panel__error" role="alert">inventory err: {inventoryError}</p>
  {/if}
  <table class="provider-inventory-panel__table">
    <thead>
      <tr>
        <th scope="col">backend</th>
        <th scope="col">display</th>
        <th scope="col">configured</th>
        <th scope="col">verified_at</th>
        <th scope="col">stale</th>
        <th scope="col">models</th>
        <th scope="col"><span class="sr-only">select</span></th>
      </tr>
    </thead>
    <tbody>
      {#each inventoryRows as row (row.backend)}
        <tr>
          <td class="mono">{row.backend}</td>
          <td>{row.display_name}</td>
          <td>{row.configured ? 'yes' : 'no'}</td>
          <td class="mono">{formatVerifiedAt(row.verified_at)}</td>
          <td>
            {#if row.stale}
              <span class="provider-inventory-panel__stub-badge" title="verified_at is missing or older than the staleness threshold">stale</span>
            {:else}
              <span class="provider-inventory-panel__dash">—</span>
            {/if}
          </td>
          <td class="provider-inventory-panel__roles">{row.models.length === 0 ? '—' : row.models.join(', ')}</td>
          <td>
            <button
              class="provider-inventory-panel__test-btn"
              type="button"
              aria-label="Select {row.backend} for BYOK configure"
              aria-pressed={byokBackend === row.backend}
              onclick={() => { void onSelectInventoryBackend(row.backend); }}
            >select</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if byokBackend}
    <div class="provider-inventory-panel__byok-form" role="group" aria-label="BYOK configure for {byokBackend}">
      <div class="provider-inventory-panel__byok-row">
        <span class="mono provider-inventory-panel__role-label">{byokBackend}</span>
        {#if byokDescriptor}
          <span class="provider-inventory-panel__hint">{byokDescriptor.display_name}</span>
        {/if}
      </div>
      {#if byokSecretKey}
        <label class="provider-inventory-panel__byok-field">
          <span class="provider-inventory-panel__role-label">{byokSecretKey.name}</span>
          <input
            class="provider-inventory-panel__role-select"
            type="password"
            autocomplete="off"
            placeholder={byokSecretKey.required ? 'required' : 'optional'}
            value={byokApiKey}
            oninput={(e) => (byokApiKey = (e.currentTarget as HTMLInputElement).value)}
          />
        </label>
      {/if}
      {#if byokShowBaseUrl}
        <label class="provider-inventory-panel__byok-field">
          <span class="provider-inventory-panel__role-label">base_url</span>
          <input
            class="provider-inventory-panel__role-select"
            type="text"
            autocomplete="off"
            placeholder="https://..."
            value={byokBaseUrl}
            oninput={(e) => (byokBaseUrl = (e.currentTarget as HTMLInputElement).value)}
          />
        </label>
      {/if}
      <label class="provider-inventory-panel__byok-field">
        <span class="provider-inventory-panel__role-label">model</span>
        <input
          class="provider-inventory-panel__role-select"
          type="text"
          autocomplete="off"
          placeholder="optional model override"
          value={byokModel}
          oninput={(e) => (byokModel = (e.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <div class="provider-inventory-panel__byok-actions">
        <button
          class="provider-inventory-panel__test-btn"
          type="button"
          disabled={byokBusy}
          onclick={() => { void onByokTest(); }}
        >{byokBusy ? 'busy…' : 'test'}</button>
        <button
          class="provider-inventory-panel__test-btn"
          type="button"
          disabled={byokBusy}
          onclick={() => { void onByokSave(); }}
        >{byokBusy ? 'busy…' : 'save'}</button>
      </div>
      {#if byokReport}
        <div
          class="provider-inventory-panel__byok-report"
          class:provider-inventory-panel__test-result--pass={byokReport.ok}
          class:provider-inventory-panel__test-result--fail={!byokReport.ok}
          role="status"
        >
          <span>{byokReport.ok ? 'ok' : 'fail'}</span>
          {#if byokReport.health}<span class="mono"> · {byokReport.health}</span>{/if}
          {#if byokReport.saved}<span> · saved</span>{/if}
          {#if byokReport.verified_at != null}<span> · verified {formatVerifiedAt(byokReport.verified_at)}</span>{/if}
          {#if byokReport.error}<span title={byokReport.error}> · err: {byokReport.error.length > 60 ? `${byokReport.error.slice(0, 60)}…` : byokReport.error}</span>{/if}
        </div>
      {/if}
    </div>
  {/if}
</section>

<style>
  .provider-inventory-panel {
    border-top: 1px solid #222;
    padding-top: 0.6rem;
    margin-top: 0.5rem;
  }
  .provider-inventory-panel h4 {
    margin: 0 0 0.25rem;
    font-size: 0.85rem;
  }
  .provider-inventory-panel__hint {
    color: #888;
    font-size: 0.75rem;
    margin: 0 0 0.5rem;
  }
  .provider-inventory-panel__error {
    color: #f88;
    margin: 0 0 0.5rem;
  }
  .provider-inventory-panel__table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 0.75rem;
  }
  .provider-inventory-panel__table th {
    text-align: left;
    color: #888;
    font-weight: 500;
    font-size: 0.7rem;
    text-transform: uppercase;
    padding: 0.2rem 0.4rem;
  }
  .provider-inventory-panel__table td {
    padding: 0.3rem 0.4rem;
    border-top: 1px solid #222;
    font-size: 0.8rem;
    vertical-align: middle;
  }
  .provider-inventory-panel__roles {
    color: #ccc;
    font-size: 0.75rem;
  }
  .provider-inventory-panel__stub-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
    background: #553;
    color: #ffc;
    border-radius: 3px;
    letter-spacing: 0.05em;
  }
  .provider-inventory-panel__dash {
    color: #555;
  }
  .provider-inventory-panel__test-btn {
    font-size: 0.7rem;
    padding: 0.15rem 0.55rem;
    background: #1f2a36;
    color: #cfe;
    border: 1px solid #2a3a4a;
    border-radius: 3px;
    cursor: pointer;
  }
  .provider-inventory-panel__test-btn:hover:not([disabled]),
  .provider-inventory-panel__test-btn:focus-visible:not([disabled]) {
    background: #2a3a4a;
    outline: none;
    border-color: #4a6a8a;
  }
  .provider-inventory-panel__test-btn[disabled] {
    opacity: 0.55;
    cursor: progress;
  }
  .provider-inventory-panel__test-result--pass {
    background: #134;
    color: #8df;
  }
  .provider-inventory-panel__test-result--fail {
    background: #411;
    color: #f88;
  }
  .provider-inventory-panel__role-label {
    font-size: 0.75rem;
    color: #cfd;
  }
  .provider-inventory-panel__role-select {
    font-size: 0.75rem;
    padding: 0.15rem 0.3rem;
    background: #181818;
    color: #eee;
    border: 1px solid #2a2a2a;
    border-radius: 3px;
    cursor: pointer;
    min-width: 0;
  }
  .provider-inventory-panel__role-select:focus-visible {
    outline: 1px solid #5af;
    outline-offset: 0;
    border-color: #5af;
  }
  .provider-inventory-panel__byok-form {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid #222;
  }
  .provider-inventory-panel__byok-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .provider-inventory-panel__byok-field {
    display: grid;
    grid-template-columns: 8rem 1fr;
    align-items: center;
    gap: 0.4rem;
  }
  .provider-inventory-panel__byok-actions {
    display: flex;
    gap: 0.4rem;
  }
  .provider-inventory-panel__byok-report {
    font-size: 0.75rem;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
