<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from './lib/state/session.svelte';
  import { reindex } from './lib/state/reindex.svelte';
  import Intro from './lib/screens/Intro.svelte';
  import ProjectSelector from './lib/screens/ProjectSelector.svelte';
  import Workspace from './lib/screens/Workspace.svelte';
  // Frameless-window resize handles (decorations:false removes native
  // edge grips on Windows). Mounted once at the root so it covers all
  // three screens; renders nothing outside the Tauri shell.
  import ResizeFrame from './lib/chrome/ResizeFrame.svelte';

  // Reindex listener is owned by the App root so it registers before any
  // ProjectSelector.openProject() fire-and-forget reindex_now() call. The
  // StatusBar (mounted only on the workspace screen) consumes the reactive
  // state instead of owning the Tauri channel — this avoids the mount race
  // where the 'started' event would fire before the listener registers.
  onMount(() => {
    reindex.bootstrap();
    // Sync persisted execution mode (localStorage) into the backend mirror —
    // backend AppState resets to LOCAL on every launch, so without this the
    // picker shows the saved mode but the provider router would use the
    // default until the user re-toggled.
    void session.reconcileMode();
    return () => reindex.teardown();
  });
</script>

<main class="app-root">
  {#if session.screen === 'intro'}
    <Intro />
  {:else if session.screen === 'selector'}
    <ProjectSelector />
  {:else}
    <Workspace />
  {/if}

  <ResizeFrame />
</main>

<style>
  .app-root {
    position: fixed;
    inset: 0;
    background: var(--bg-void);
    font-family: var(--font-sans);
    color: var(--text-primary);
    overflow: hidden;
  }
</style>
