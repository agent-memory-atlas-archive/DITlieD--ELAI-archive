<script lang="ts">
  // B-35 Phase C — Context Artifacts panel.
  //
  // Mounts inside KnowledgeDrawer.svelte. Lists indexed artifacts from the
  // `list_artifacts` Tauri command. Supports inline search via `search_artifacts`
  // and per-artifact re-index via `index_artifacts`.
  //
  // Required Tauri command interface (to be registered in src-tauri/lib.rs):
  //   list_artifacts({ db_path: string }) -> { artifact_names: string[] }
  //   search_artifacts({ db_path, query, artifact_name?, limit? })
  //     -> { results: ArtifactSearchResult[] }
  //   index_artifacts({ db_path, artifact_name: string }) -> { ok: true, artifact_name: string, chunks_dropped: number }
  //   remove_artifacts({ db_path, artifact_name: string }) -> void

  import { onMount } from 'svelte';
  import { Card, Badge, Button, Input } from '$lib/ui';

  type ArtifactRecord = {
    name: string;
  };

  type ArtifactSearchResult = {
    artifact_name: string;
    relative_path: string;
    content: string;
    start_line: number;
    end_line: number;
    score: number;
  };

  // db_path must be passed in from the parent (KnowledgeDrawer has the project context).
  // When undefined, the panel renders a "no project" notice.
  let { db_path = '' }: { db_path?: string } = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let artifacts = $state<ArtifactRecord[]>([]);
  let searchQuery = $state('');
  let searchResults = $state<ArtifactSearchResult[]>([]);
  let selectedArtifact = $state<string | null>(null);
  let loading = $state(false);
  let indexingArtifact = $state<string | null>(null);
  let lastError = $state<string | null>(null);
  let searchInFlight = $state(false);
  // WARN-5: inline status shown after index_artifacts completes.
  let indexStatus = $state<string | null>(null);
  let tauriInvoke: (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | null = null;

  // ---------------------------------------------------------------------------
  // Tauri lazy-import
  // ---------------------------------------------------------------------------

  async function ensureInvoke(): Promise<typeof tauriInvoke> {
    if (tauriInvoke !== null) return tauriInvoke;
    const w = typeof window !== 'undefined' ? (window as unknown as Record<string, unknown>) : {};
    if (!w.__TAURI_INTERNALS__ && !w.__TAURI__) {
      return null;
    }
    try {
      const mod = await import('@tauri-apps/api/core');
      tauriInvoke = mod.invoke;
    } catch (err) {
      console.warn('[ArtifactsPanel] @tauri-apps/api/core import failed', err);
      tauriInvoke = null;
    }
    return tauriInvoke;
  }

  // ---------------------------------------------------------------------------
  // Data fetching
  // ---------------------------------------------------------------------------

  async function loadArtifacts(): Promise<void> {
    if (!db_path) return;
    loading = true;
    lastError = null;
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        artifacts = [];
        return;
      }
      const result = await inv<{ artifact_names: string[] }>('list_artifacts', { db_path });
      artifacts = (result.artifact_names ?? []).map((name) => ({ name }));
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
      console.debug('[ArtifactsPanel] list_artifacts failed', err);
    } finally {
      loading = false;
    }
  }

  async function runSearch(): Promise<void> {
    if (!db_path || !searchQuery.trim()) {
      searchResults = [];
      return;
    }
    searchInFlight = true;
    lastError = null;
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        searchResults = [];
        return;
      }
      const args: Record<string, unknown> = {
        db_path,
        query: searchQuery.trim(),
        limit: 10,
      };
      if (selectedArtifact) {
        args.artifact_name = selectedArtifact;
      }
      const result = await inv<{ results: ArtifactSearchResult[] }>('search_artifacts', args);
      searchResults = result.results ?? [];
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
      console.debug('[ArtifactsPanel] search_artifacts failed', err);
    } finally {
      searchInFlight = false;
    }
  }

  async function reindexArtifact(name: string): Promise<void> {
    if (!db_path || indexingArtifact) return;
    indexingArtifact = name;
    lastError = null;
    indexStatus = null;
    try {
      const inv = await ensureInvoke();
      if (!inv) return;
      const result = await inv<{ ok: boolean; artifact_name: string; chunks_dropped: number }>(
        'index_artifacts',
        { db_path, artifact_name: name }
      );
      // WARN-5: surface success signal to user.
      indexStatus = `Re-indexed "${result.artifact_name}" (${result.chunks_dropped} chunks cleared)`;
      await loadArtifacts();
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
      console.debug('[ArtifactsPanel] index_artifacts failed', err);
    } finally {
      indexingArtifact = null;
    }
  }

  function toggleArtifactFilter(name: string): void {
    selectedArtifact = selectedArtifact === name ? null : name;
    // Clear results when filter changes so stale results are not shown.
    searchResults = [];
  }

  function excerptContent(content: string): string {
    return content.length > 200 ? content.slice(0, 200) + '…' : content;
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    void loadArtifacts();
  });
</script>

<Card variant="plain" padding={12}>
  <div class="section-label">CONTEXT ARTIFACTS</div>

  {#if !db_path}
    <span class="ghost-text">No project selected — open a project to view artifacts</span>
  {:else if loading}
    <span class="ghost-text">loading…</span>
  {:else if artifacts.length === 0}
    <span class="ghost-text">no artifacts configured</span>
  {:else}
    <div class="artifact-list">
      {#each artifacts as artifact}
        {@const isFiltered = selectedArtifact === artifact.name}
        {@const isIndexing = indexingArtifact === artifact.name}
        <div class="artifact-row" class:artifact-row--selected={isFiltered}>
          <span class="artifact-name">{artifact.name}</span>
          <div class="artifact-actions">
            <Button
              variant="ghost"
              size="sm"
              disabled={isIndexing}
              onclick={() => void reindexArtifact(artifact.name)}
            >
              {isIndexing ? 'indexing…' : 'Re-index'}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => toggleArtifactFilter(artifact.name)}
            >
              {isFiltered ? 'all' : 'filter'}
            </Button>
          </div>
        </div>
      {/each}
    </div>

    <!-- Search bar -->
    <div class="search-row">
      <Input
        bind:value={searchQuery}
        placeholder={selectedArtifact ? `search in ${selectedArtifact}…` : 'search all artifacts…'}
        onenter={() => void runSearch()}
      />
      <Button
        variant="ghost"
        size="sm"
        disabled={searchInFlight || !searchQuery.trim()}
        onclick={() => void runSearch()}
      >
        {searchInFlight ? '…' : 'Search'}
      </Button>
    </div>

    <!-- Search results -->
    {#if searchResults.length > 0}
      <div class="results-list">
        {#each searchResults as result}
          <Card variant="plain" padding={8}>
            <div class="result-header">
              <Badge tone="ghost" size="xs">{result.artifact_name}</Badge>
              <span class="result-location">{result.relative_path}:{result.start_line}</span>
              <span class="result-score">{result.score.toFixed(3)}</span>
            </div>
            <div class="result-content">{excerptContent(result.content)}</div>
          </Card>
        {/each}
      </div>
    {:else if searchQuery && !searchInFlight}
      <span class="ghost-text">no results</span>
    {/if}
  {/if}

  {#if lastError !== null}
    <div class="error-row">error: {lastError}</div>
  {/if}
  {#if indexStatus !== null}
    <div class="status-row">{indexStatus}</div>
  {/if}
</Card>

<!-- Refresh button row -->
<div class="refresh-row">
  <Button variant="ghost" size="sm" disabled={loading} onclick={() => void loadArtifacts()}>
    Refresh
  </Button>
</div>

<style>
  .section-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  .ghost-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .artifact-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
  }

  .artifact-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px;
    background: var(--bg-input);
    border-radius: var(--r-xs);
    gap: 8px;
  }

  .artifact-row--selected {
    border: 1px solid var(--teal-line, #2a4a4a);
  }

  .artifact-name {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .artifact-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .search-row {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-bottom: 8px;
  }

  .results-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }

  .result-location {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-score {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  .result-content {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 60px;
    overflow: hidden;
  }

  .error-row {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    margin-top: 4px;
  }

  .status-row {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    margin-top: 4px;
  }

  .refresh-row {
    display: flex;
    justify-content: flex-end;
    margin-top: 2px;
  }
</style>
