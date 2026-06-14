<!-- TopRightControls.svelte — the shared top-right control cluster:
     the sun/moon theme toggle + a hairline separator + the window
     controls (minimize / restore / close). One component so the cluster
     is IDENTICAL on the project-selector topbar and the workflow shell,
     matching the Intro hero's .tr-toggles layout (DESIGN §12 + §13). -->
<script lang="ts">
  import ThemeToggle from './ThemeToggle.svelte';
  import WindowControls from './WindowControls.svelte';
</script>

<div class="tr-controls">
  <ThemeToggle />
  <span class="wc-sep" aria-hidden="true"></span>
  <WindowControls />
</div>

<style>
  .tr-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }
  /* hairline divider before the window controls (matches Intro) */
  .wc-sep {
    width: 1px;
    height: 18px;
    margin: 0 2px;
    background: var(--line);
  }
</style>
