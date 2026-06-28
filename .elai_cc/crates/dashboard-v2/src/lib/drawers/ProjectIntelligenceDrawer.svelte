<script lang="ts">
  import { onMount } from 'svelte';
  import { projectIntelligenceData } from '$lib/state/drawers.svelte';
  import Badge from '$lib/ui/Badge.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Divider from '$lib/ui/Divider.svelte';
  import UnwiredNote from '$lib/ui/UnwiredNote.svelte';
  import { logStub } from '$lib/api/stub';

  onMount(() => {
    projectIntelligenceData.bootstrap();
    return projectIntelligenceData.teardown;
  });

  const profile = $derived(projectIntelligenceData.profile);

  const tier1Rows = $derived(() => [
    { label: 'language',     value: profile.tier1.language },
    { label: 'build system', value: profile.tier1.build_tool },
    { label: 'framework',    value: profile.tier1.framework },
    { label: 'test runner',  value: profile.tier1.test_runner },
    { label: 'ci',           value: profile.tier1.ci },
    { label: 'linter',       value: profile.tier1.linter },
  ]);

  const levelLabels: Record<number, string> = {
    1: 'PROJECT',
    2: 'CONCEPT',
    3: 'PHASE',
    4: 'STEP',
  };

  let overrideOpen = $state(false);
  let overrideText = $state(projectIntelligenceData.profile.override_file ?? '');

  function toggleOverride() {
    overrideOpen = !overrideOpen;
  }

  function saveOverride() {
    logStub('project_intelligence.save_override', { text: overrideText });
  }

  function strengthTone(s: string): string {
    if (s === 'strong') return 'teal';
    if (s === 'moderate') return 'warn';
    return 'ghost';
  }

  function strengthLabel(s: string): string {
    return s.toUpperCase();
  }
</script>

<div class="drawer">
  <!-- TIER 1 -->
  <section class="section">
    <h3 class="section-header">TIER 1 — STRUCTURAL</h3>
    <div class="tier1-grid">
      {#each tier1Rows() as row}
        <span class="label">{row.label}</span>
        <span class="value">{row.value}</span>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- TIER 2 -->
  <section class="section">
    <h3 class="section-header">TIER 2 — CONVENTIONS</h3>
    <div class="convention-list">
      {#each profile.tier2.conventions as conv}
        <div class="convention-row">
          <span class="conv-name">{conv.name}</span>
          <Badge tone={strengthTone(conv.strength)} size="xs">{strengthLabel(conv.strength)}</Badge>
        </div>
        <div class="conv-example">{conv.example}</div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- TIER 3 -->
  <section class="section">
    <h3 class="section-header">TIER 3 — TELESCOPE</h3>
    <div class="telescope-stack">
      {#each profile.tier3.telescope_summaries as item}
        <div class="telescope-card">
          <span class="tele-level">{levelLabels[item.level] ?? `LEVEL ${item.level}`}</span>
          <p class="tele-summary">{item.summary}</p>
        </div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- OVERRIDE FILE -->
  <section class="section">
    <h3 class="section-header">OVERRIDE FILE</h3>
    {#if projectIntelligenceData.unwired.override}
      <UnwiredNote
        phase="A-32"
        feature="manual ProjectProfile override TOML editor + save"
        note="Save is logStub only; backend command save_project_intelligence_override not yet implemented"
      />
    {/if}
    <div class="override-row">
      <Button variant="ghost" size="sm" onclick={toggleOverride}>
        {overrideOpen ? 'Close override' : 'Edit override'}
      </Button>
    </div>
    {#if overrideOpen}
      <div class="override-area">
        <textarea
          class="override-textarea"
          rows={10}
          bind:value={overrideText}
          placeholder="# Paste override TOML / JSON here..."
        ></textarea>
        <div class="override-save-row">
          <Button variant="ghost" size="sm" onclick={saveOverride}>Save (stub)</Button>
        </div>
      </div>
    {/if}
  </section>
</div>

<style>
  .drawer {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-bottom: var(--s-6);
  }

  .section {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }

  .section-header {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    text-transform: uppercase;
    margin: 0 0 var(--s-3) 0;
  }

  .tier1-grid {
    display: grid;
    grid-template-columns: 9ch 1fr;
    row-gap: 6px;
    column-gap: var(--s-3);
  }

  .label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    word-break: break-word;
  }

  .convention-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .convention-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-2);
  }

  .conv-name {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .conv-example {
    font-family: var(--font-sans);
    font-size: 10px;
    color: var(--text-ghost);
    margin-bottom: var(--s-1);
    padding-left: 2px;
  }

  .telescope-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .telescope-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tele-level {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    text-transform: uppercase;
  }

  .tele-summary {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: var(--leading-text);
    margin: 0;
  }

  .override-row {
    margin-bottom: var(--s-2);
  }

  .override-area {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    margin-top: var(--s-2);
  }

  .override-textarea {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: var(--leading-code);
    padding: var(--s-2) var(--s-3);
    resize: vertical;
    outline: none;
    box-sizing: border-box;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .override-textarea:focus {
    border-color: var(--teal-line);
    box-shadow: var(--ring);
  }

  .override-save-row {
    display: flex;
    justify-content: flex-end;
  }
</style>
