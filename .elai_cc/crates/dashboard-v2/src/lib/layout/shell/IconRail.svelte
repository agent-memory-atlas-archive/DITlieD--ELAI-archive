<script lang="ts">
  // IconRail.svelte — thin (~48px) persistent left rail, the "side menu"
  // (VS-Code activity-bar idiom). DESIGN custom HAIRLINE icons (1px stroke,
  // one teal accent on active; NO generic icon-tiles per DESIGN §10). It is
  // the TRIGGER for the zero-width flyout (DESIGN §8 reconciliation note in
  // the brief §7): the rail is persistent + thin, the explorer/menu content
  // is the flyout (zero-width until summoned).
  //
  // Carries: a MENU/flyout toggle (project intelligence + monitor long-tail
  // + settings entry live INSIDE the flyout) and a FILES toggle (same flyout,
  // scrolled to the file tree). Both open the one MenuFlyout.
  import { shell } from '../../state/shell.svelte';

  interface Props {
    /** Open the flyout focused on the menu section. */
    onMenu: () => void;
    /** Open the flyout focused on the files section. */
    onFiles: () => void;
  }
  let { onMenu, onFiles }: Props = $props();
</script>

<nav class="icon-rail" aria-label="Workspace rail">
  <!-- MENU / flyout toggle. Hairline "stacked rows" glyph. -->
  <button
    class="rail-btn"
    class:rail-btn--active={shell.flyoutOpen}
    onclick={onMenu}
    aria-label="Menu and files"
    title="Menu / files"
  >
    {#if shell.flyoutOpen}<span class="active-bar" aria-hidden="true"></span>{/if}
    <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
      <line x1="5" y1="8" x2="19" y2="8" />
      <line x1="5" y1="12" x2="19" y2="12" />
      <line x1="5" y1="16" x2="19" y2="16" />
    </svg>
  </button>

  <!-- FILES toggle. Hairline folder glyph. -->
  <button
    class="rail-btn"
    onclick={onFiles}
    aria-label="File explorer"
    title="Files"
  >
    <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 7 L9 7 L11 9.3 L20 9.3 L20 18 L4 18 Z" />
    </svg>
  </button>

  <span class="rail-spacer" aria-hidden="true"></span>

  <!-- SETTINGS entry. Opens the settings modal directly. Hairline gear-ring. -->
  <button
    class="rail-btn rail-btn--bottom"
    onclick={() => shell.openSettings()}
    aria-label="Settings"
    title="Settings"
  >
    <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="3.2" />
      <circle cx="12" cy="12" r="7.4" stroke-dasharray="2.1 2.6" />
    </svg>
  </button>
</nav>

<style>
  .icon-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 48px;
    height: 100%;
    background: var(--bg);
    border-right: 1px solid var(--line);
    /* leave space below the top-left window-control drag strip */
    padding: 6px 0 8px;
    gap: 2px;
  }

  .rail-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 44px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--dim);
    outline: none;
    transition: color var(--dur-fast) var(--ease-out);
  }

  .rail-btn:hover { color: var(--text); }
  .rail-btn--active { color: var(--d-teal-bright); }
  .rail-btn:focus-visible { box-shadow: inset 0 0 0 1px var(--d-teal); }

  .rail-btn--bottom { margin-top: auto; }
  .rail-spacer { flex: 1 1 auto; }

  .active-bar {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 2px;
    height: 22px;
    background: var(--d-teal);
    border-radius: 0 2px 2px 0;
  }

  .glyph {
    width: 20px;
    height: 20px;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-linejoin: round;
    fill: none;
  }
</style>
