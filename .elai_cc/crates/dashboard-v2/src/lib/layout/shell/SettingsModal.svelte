<script lang="ts">
  // SettingsModal.svelte — Settings as a POPUP MODAL (DESIGN §8 semantic
  // z-scale), NOT a window/tab. Wraps the EXISTING SettingsWindow so all its
  // settings wiring (per-role runtime, routing, gates, UX, motion, etc.)
  // stays intact — only the SURFACING changes from a window to a modal.
  import { shell } from '../../state/shell.svelte';
  import SettingsWindow from '../../windows/SettingsWindow.svelte';
</script>

{#if shell.settingsOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="modal-scrim"
    onclick={(e) => { if (e.target === e.currentTarget) shell.closeSettings(); }}
  >
    <div class="modal" role="dialog" aria-modal="true" aria-label="Settings">
      <div class="modal-head" data-tauri-drag-region>
        <span class="mt">SETTINGS</span>
        <button class="modal-close" onclick={() => shell.closeSettings()} aria-label="Close settings">✕</button>
      </div>
      <div class="modal-body">
        <SettingsWindow />
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-scrim {
    position: fixed;
    inset: 0;
    z-index: var(--z-shell-modal, 80);
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
  }

  .modal {
    width: min(900px, 92vw);
    height: min(620px, 86vh);
    background: var(--panel);
    border: 1px solid var(--line2);
    border-radius: var(--r-lg);
    overflow: hidden;
    display: grid;
    grid-template-rows: 46px 1fr;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
  }

  .modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
  }
  .mt {
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.2em;
    color: var(--text);
  }
  .modal-close {
    background: none;
    border: none;
    color: var(--dim);
    font-size: 15px;
    cursor: pointer;
    line-height: 1;
    padding: 4px 6px;
    border-radius: var(--r-sm);
    transition: color var(--dur-fast) var(--ease-out);
  }
  .modal-close:hover { color: var(--d-teal-bright); }
  .modal-close:focus-visible { box-shadow: inset 0 0 0 1px var(--d-teal); }

  .modal-body {
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
