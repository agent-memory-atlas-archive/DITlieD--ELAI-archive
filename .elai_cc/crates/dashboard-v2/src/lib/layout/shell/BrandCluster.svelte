<!-- BrandCluster.svelte — the shared top-left brand lockup: a hairline
     back-chevron (center->out underline idiom) + the shared ELAI text
     Wordmark. ONE component used by BOTH the project-selector topbar and
     the workflow TabBar so the back button + wordmark are pixel-identical
     in size, spacing and position, and never drift between screens
     (DESIGN §16). Page-specific bits (the selector's "· session select"
     subtitle) sit AFTER this component, never inside it. -->
<script lang="ts">
  import Wordmark from './Wordmark.svelte';

  interface Props {
    /** where the back chevron navigates (selector -> intro; shell -> selector) */
    onBack: () => void;
    /** accessible label + tooltip for the back chevron */
    backLabel: string;
  }
  let { onBack, backLabel }: Props = $props();
</script>

<div class="brand-cluster">
  <button
    class="back-btn"
    type="button"
    onclick={onBack}
    aria-label={backLabel}
    title={backLabel}
  >
    <svg class="back-glyph" viewBox="0 0 16 16" aria-hidden="true">
      <polyline points="10,3 5,8 10,13" />
    </svg>
  </button>
  <Wordmark />
</div>

<style>
  .brand-cluster {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    -webkit-app-region: no-drag;
  }

  /* back chevron — hairline, teal on hover, center->out underline idiom
     (DESIGN §13: no dot, no box) */
  .back-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--ghost);
    padding: 0;
    transition: color var(--dur-fast) var(--ease-out);
    outline: none;
  }
  .back-btn::after {
    content: '';
    position: absolute;
    left: 3px;
    right: 3px;
    bottom: 2px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .back-btn:hover {
    color: var(--d-teal-bright);
  }
  .back-btn:hover::after {
    transform: scaleX(1);
  }
  .back-btn:focus-visible {
    box-shadow: inset 0 0 0 1px var(--d-teal);
    border-radius: var(--r-sm);
  }
  .back-glyph {
    width: 14px;
    height: 14px;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
    fill: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .back-btn::after {
      transition: none;
    }
  }
</style>
