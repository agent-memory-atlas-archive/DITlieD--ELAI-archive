<script lang="ts">
  // ============================================================
  // ResizeFrame.svelte — invisible edge/corner resize handles for the
  // frameless (decorations:false) Tauri window.
  //
  // On Windows, decorations:false removes the native resize grips, so
  // an edge drag no longer resizes the window. This overlays 8 thin
  // (~5px) transparent handles around the app root that call
  // getCurrentWindow().startResizeDragging(<ResizeDirection>) on
  // pointer-down. ResizeDirection is the string-literal union from
  // @tauri-apps/api/window 2.x ('North' | 'NorthEast' | ... | 'West'),
  // verified against the installed window.d.ts.
  //
  // Browser-dev safe: renders NOTHING when not inside the Tauri shell
  // (same __TAURI_INTERNALS__ detection stub.ts uses), so the browser
  // owns its own resizing and getCurrentWindow() is never touched.
  //
  // Harmless when native resize is ALSO present: the handles are a thin
  // transparent border ring that only acts on pointer-down; if the OS
  // still draws grips they simply overlap with no visual change.
  // ============================================================

  // ResizeDirection literal union (verified in window.d.ts:42).
  type Dir =
    | 'North'
    | 'NorthEast'
    | 'East'
    | 'SouthEast'
    | 'South'
    | 'SouthWest'
    | 'West'
    | 'NorthWest';

  const IN_TAURI: boolean =
    typeof window !== 'undefined' &&
    (Boolean((window as any).__TAURI_INTERNALS__) ||
      Boolean((window as any).__TAURI__));

  async function startResize(e: PointerEvent, dir: Dir) {
    if (!IN_TAURI) return;
    // Only primary button initiates a resize drag.
    if (e.button !== 0) return;
    e.preventDefault();
    try {
      const mod = await import('@tauri-apps/api/window');
      await mod.getCurrentWindow().startResizeDragging(dir);
    } catch (err) {
      console.warn('[ResizeFrame] startResizeDragging failed', err);
    }
  }

  // edge/corner descriptors: [class, direction]
  const HANDLES: [string, Dir][] = [
    ['n', 'North'],
    ['s', 'South'],
    ['e', 'East'],
    ['w', 'West'],
    ['ne', 'NorthEast'],
    ['nw', 'NorthWest'],
    ['se', 'SouthEast'],
    ['sw', 'SouthWest'],
  ];
</script>

{#if IN_TAURI}
  <div class="resize-frame" aria-hidden="true">
    {#each HANDLES as [cls, dir]}
      <!-- decorative resize grip; keyboard users resize via the OS / the
           maximize control, so no interactive role is needed -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class={`rh rh-${cls}`}
        role="presentation"
        onpointerdown={(e) => startResize(e, dir)}
      ></div>
    {/each}
  </div>
{/if}

<style>
  /* The frame itself never blocks pointer events; only the handles do. */
  .resize-frame {
    position: fixed;
    inset: 0;
    z-index: 9999; /* above all app chrome so an edge drag always wins */
    pointer-events: none;
  }
  .rh {
    position: absolute;
    pointer-events: auto;
  }

  /* 5px edges */
  .rh-n {
    top: 0;
    left: 6px;
    right: 6px;
    height: 5px;
    cursor: ns-resize;
  }
  .rh-s {
    bottom: 0;
    left: 6px;
    right: 6px;
    height: 5px;
    cursor: ns-resize;
  }
  .rh-e {
    top: 6px;
    bottom: 6px;
    right: 0;
    width: 5px;
    cursor: ew-resize;
  }
  .rh-w {
    top: 6px;
    bottom: 6px;
    left: 0;
    width: 5px;
    cursor: ew-resize;
  }

  /* 8px corners (sit above the edges) */
  .rh-ne {
    top: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }
  .rh-nw {
    top: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
  }
  .rh-se {
    bottom: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
  }
  .rh-sw {
    bottom: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }
</style>
