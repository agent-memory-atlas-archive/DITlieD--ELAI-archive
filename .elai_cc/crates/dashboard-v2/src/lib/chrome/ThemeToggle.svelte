<!-- ThemeToggle.svelte — shared sun/moon theme toggle (DESIGN §12).
     Present on EVERY screen top-right. Resting icon = current mode;
     hover cross-fades to the icon the click will switch to
     (dark: moon at rest -> sun on hover; light: sun at rest -> moon on hover).
     prefers-reduced-motion: instant swap via tokens.css --dur-base -> 0ms.

     Uses the existing theme store. NO new dependency.
-->
<script lang="ts">
  import { theme } from '../state/theme.svelte';

  function toggleTheme() {
    theme.toggle();
  }
</script>

<button
  class="icon-toggle"
  type="button"
  title="Toggle light / dark"
  aria-label="Toggle light / dark mode"
  onclick={toggleTheme}
>
  <svg class="ico ico-sun" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="4" />
    <line x1="12" y1="2.5" x2="12" y2="5"    />
    <line x1="12" y1="19"  x2="12" y2="21.5" />
    <line x1="2.5" y1="12" x2="5"   y2="12"  />
    <line x1="19"  y1="12" x2="21.5" y2="12" />
    <line x1="5.2"  y1="5.2"  x2="6.9"  y2="6.9"  />
    <line x1="17.1" y1="17.1" x2="18.8" y2="18.8" />
    <line x1="18.8" y1="5.2"  x2="17.1" y2="6.9"  />
    <line x1="6.9"  y1="17.1" x2="5.2"  y2="18.8" />
  </svg>
  <svg class="ico ico-moon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M20 14.6A8 8 0 0 1 9.4 4 7 7 0 1 0 20 14.6Z" />
  </svg>
</button>

<style>
  .icon-toggle {
    appearance: none;
    cursor: pointer;
    position: relative;
    width: 33px;
    height: 31px;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 0;
    color: var(--read);
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .icon-toggle:hover {
    color: var(--d-teal);
    border-color: var(--d-teal-soft);
  }
  .icon-toggle:focus-visible {
    box-shadow: inset 0 0 0 1px var(--d-teal);
  }

  .ico {
    position: absolute;
    top: 50%;
    left: 50%;
    transition:
      opacity var(--dur-base) var(--ease-out),
      transform var(--dur-base) var(--ease-out);
  }

  /* dark (default): moon at rest, sun hidden */
  .ico-moon {
    opacity: 1;
    transform: translate(-50%, -50%) rotate(0deg) scale(1);
  }
  .ico-sun {
    opacity: 0;
    transform: translate(-50%, -50%) rotate(-70deg) scale(0.5);
  }

  /* dark hover: moon exits, sun enters */
  .icon-toggle:hover .ico-moon {
    opacity: 0;
    transform: translate(-50%, -50%) rotate(70deg) scale(0.5);
  }
  .icon-toggle:hover .ico-sun {
    opacity: 1;
    transform: translate(-50%, -50%) rotate(0deg) scale(1);
  }

  /* light theme: sun at rest, moon hidden */
  :global(:root[data-theme='light']) .ico-sun {
    opacity: 1;
    transform: translate(-50%, -50%) rotate(0deg) scale(1);
  }
  :global(:root[data-theme='light']) .ico-moon {
    opacity: 0;
    transform: translate(-50%, -50%) rotate(-70deg) scale(0.5);
  }

  /* light hover: sun exits, moon enters */
  :global(:root[data-theme='light']) .icon-toggle:hover .ico-sun {
    opacity: 0;
    transform: translate(-50%, -50%) rotate(70deg) scale(0.5);
  }
  :global(:root[data-theme='light']) .icon-toggle:hover .ico-moon {
    opacity: 1;
    transform: translate(-50%, -50%) rotate(0deg) scale(1);
  }
</style>
