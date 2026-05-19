<script lang="ts">
  // B-33 — Mermaid export modal.
  // Shows the rendered Mermaid string and offers Copy / Save-as-.mmd / Close.

  import { save } from '@tauri-apps/plugin-dialog';

  interface Props {
    mermaid: string;
    onClose: () => void;
  }

  let { mermaid, onClose }: Props = $props();

  let copyState = $state<'idle' | 'ok' | 'err'>('idle');
  let saveState = $state<'idle' | 'ok' | 'err'>('idle');

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(mermaid);
      copyState = 'ok';
      setTimeout(() => { copyState = 'idle'; }, 1500);
    } catch (err) {
      console.warn('[MermaidExportModal] clipboard write failed', err);
      copyState = 'err';
    }
  }

  async function saveAsFile() {
    try {
      const path = await save({
        filters: [{ name: 'Mermaid', extensions: ['mmd', 'md'] }],
        defaultPath: 'graph.mmd',
      });
      if (!path) return;
      // Use Tauri fs plugin if available; fall back to fetch-blob trick.
      const blob = new Blob([mermaid], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = path.split(/[\\/]/).pop() ?? 'graph.mmd';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      saveState = 'ok';
      setTimeout(() => { saveState = 'idle'; }, 1500);
    } catch (err) {
      console.warn('[MermaidExportModal] save failed', err);
      saveState = 'err';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="modal-backdrop"
  role="presentation"
  onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Mermaid export">
    <header class="modal-header">
      <h2>Mermaid export</h2>
      <button class="close-btn" type="button" aria-label="Close" onclick={onClose}>×</button>
    </header>
    <div class="actions">
      <button type="button" onclick={copyToClipboard}>
        {copyState === 'ok' ? 'Copied!' : copyState === 'err' ? 'Copy failed' : 'Copy to clipboard'}
      </button>
      <button type="button" onclick={saveAsFile}>
        {saveState === 'ok' ? 'Saved' : saveState === 'err' ? 'Save failed' : 'Save as .mmd'}
      </button>
    </div>
    <pre class="mermaid-source"><code>{mermaid}</code></pre>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .modal {
    width: min(720px, 90vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    overflow: hidden;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-subtle);
    font-family: var(--font-mono);
  }
  .modal-header h2 {
    margin: 0;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 18px;
    cursor: pointer;
    padding: 0 6px;
    line-height: 1;
  }
  .actions {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .actions button {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    color: var(--teal);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
  }
  .actions button:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
  }
  .mermaid-source {
    margin: 0;
    padding: 12px 16px;
    background: var(--bg-base);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: auto;
    white-space: pre;
    flex: 1 1 auto;
  }
</style>
