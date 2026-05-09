<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { bootRemoteSidecar } from '../api/stub';
  import type { RemoteSidecarProgress, RemoteSidecarStep, RemoteSidecarStepStatus } from '../api/types';

  interface Props {
    onDone?: () => void;
    onSkip?: () => void;
  }

  let { onDone, onSkip }: Props = $props();

  // ---------------------------------------------------------------------------
  // Step definitions (display order)
  // ---------------------------------------------------------------------------
  const STEP_DEFS: { key: RemoteSidecarStep; label: string }[] = [
    { key: 'ssh',    label: 'SSH Boot' },
    { key: 'tunnel', label: 'SSH Tunnel' },
    { key: 'health', label: 'Health Check' },
    { key: 'done',   label: 'Dashboard' },
  ];

  // ---------------------------------------------------------------------------
  // Reactive state
  // ---------------------------------------------------------------------------
  type StepState = 'ghost' | 'running' | 'ok' | 'error';

  interface StepEntry {
    key: RemoteSidecarStep;
    label: string;
    state: StepState;
    error?: string;
  }

  let steps: StepEntry[] = $state(
    STEP_DEFS.map((d) => ({ key: d.key, label: d.label, state: 'ghost' as StepState }))
  );

  let dismissed = $state(false);

  function progressStatusToState(status: RemoteSidecarStepStatus): StepState {
    switch (status) {
      case 'running':  return 'running';
      case 'ok':       return 'ok';
      case 'error':    return 'error';
      default:         return 'ghost';
    }
  }

  function applyProgress(p: RemoteSidecarProgress) {
    steps = steps.map((s) => {
      if (s.key !== p.step) return s;
      return { ...s, state: progressStatusToState(p.status), error: p.error };
    });
    if (p.step === 'done' && p.status === 'ok') {
      // Brief pause so user sees the completed state before auto-dismiss.
      setTimeout(() => { dismissed = true; onDone?.(); }, 800);
    }
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await bootRemoteSidecar(applyProgress);
  });

  onDestroy(() => {
    unlisten?.();
  });

  function handleSkip() {
    dismissed = true;
    onSkip?.();
  }
</script>

{#if !dismissed}
  <div class="overlay-backdrop" role="presentation">
    <div class="overlay" role="dialog" aria-modal="true" aria-label="Remote sidecar boot">
      <div class="header">
        <span class="title">CONNECTING TO GPU BOX</span>
        <button class="skip-btn" onclick={handleSkip} aria-label="Skip boot overlay">
          SKIP
        </button>
      </div>

      <div class="steps">
        {#each steps as step (step.key)}
          <div class="step" data-state={step.state}>
            <div class="step-indicator">
              {#if step.state === 'ghost'}
                <span class="indicator indicator--ghost" aria-hidden="true"></span>
              {:else if step.state === 'running'}
                <span class="indicator indicator--breath" aria-hidden="true"></span>
              {:else if step.state === 'ok'}
                <span class="indicator indicator--ok" aria-hidden="true"></span>
              {:else}
                <span class="indicator indicator--error" aria-hidden="true"></span>
              {/if}
            </div>
            <span class="step-label">{step.label}</span>
            <span class="step-status">
              {#if step.state === 'ghost'}
                —
              {:else if step.state === 'running'}
                running
              {:else if step.state === 'ok'}
                ok
              {:else}
                error
              {/if}
            </span>
            {#if step.error}
              <span class="step-error" title={step.error}>{step.error.slice(0, 80)}</span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    /* Semi-transparent dark backdrop, no blur per DESIGN.md flat aesthetic */
    background: rgba(10, 12, 16, 0.82);
    z-index: 9000;
  }

  .overlay {
    background: var(--color-surface, #0d1117);
    border: 1px solid var(--color-border, #21262d);
    min-width: 340px;
    max-width: 480px;
    width: 100%;
    padding: 20px 24px 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* ---- header ---- */
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .title {
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--color-muted, #6e7681);
    text-transform: uppercase;
  }

  .skip-btn {
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--color-muted, #6e7681);
    background: none;
    border: 1px solid var(--color-border, #21262d);
    padding: 3px 8px;
    cursor: pointer;
    text-transform: uppercase;
    transition: color 150ms, border-color 150ms;
  }

  .skip-btn:hover {
    color: var(--color-fg, #e6edf3);
    border-color: var(--color-muted, #6e7681);
  }

  /* ---- steps ---- */
  .steps {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .step {
    display: grid;
    grid-template-columns: 16px 1fr auto;
    grid-template-areas: 'ind label status' 'ind error error';
    align-items: center;
    gap: 0 10px;
  }

  .step-indicator {
    grid-area: ind;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 16px;
  }

  .step-label {
    grid-area: label;
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 12px;
    color: var(--color-fg, #e6edf3);
  }

  .step[data-state='ghost'] .step-label {
    color: var(--color-muted, #6e7681);
  }

  .step-status {
    grid-area: status;
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 10px;
    color: var(--color-muted, #6e7681);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .step[data-state='ok'] .step-status {
    color: var(--color-teal, #39d353);
  }

  .step[data-state='error'] .step-status {
    color: var(--color-red, #f85149);
  }

  .step-error {
    grid-area: error;
    grid-column: 2 / span 2;
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 10px;
    color: var(--color-red, #f85149);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-top: 2px;
  }

  /* ---- indicators ---- */
  .indicator {
    display: block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  /* ghost: hollow circle outline */
  .indicator--ghost {
    border: 1px solid var(--color-muted, #6e7681);
    background: none;
  }

  /* running: breath animation (pulse) — no spinner per DESIGN.md */
  .indicator--breath {
    background: var(--color-teal, #39d353);
    animation: breath 1.4s ease-in-out infinite;
  }

  @keyframes breath {
    0%, 100% { opacity: 0.25; transform: scale(0.75); }
    50%       { opacity: 1;    transform: scale(1.1); }
  }

  /* ok: solid teal fill */
  .indicator--ok {
    background: var(--color-teal, #39d353);
  }

  /* error: solid red fill */
  .indicator--error {
    background: var(--color-red, #f85149);
  }
</style>
