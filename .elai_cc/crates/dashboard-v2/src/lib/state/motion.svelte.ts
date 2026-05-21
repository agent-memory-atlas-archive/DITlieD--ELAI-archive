// E-33: prefers-reduced-motion observer. Single source of truth so the
// orbit layer (which uses SVG <animateTransform>, untouched by CSS
// animation overrides) can opt out by guarding with {#if motion.allow}.
// CSS-only animations are halted via the @media (prefers-reduced-motion)
// block in tokens.css; this store covers JS/SVG animation that CSS misses.

const MEDIA_QUERY = '(prefers-reduced-motion: reduce)';

function createMotion() {
  let prefersReduced = $state(false);
  let userOverride = $state<'auto' | 'reduced' | 'full'>('auto');

  function detect() {
    if (typeof window === 'undefined' || !window.matchMedia) return;
    prefersReduced = window.matchMedia(MEDIA_QUERY).matches;
  }

  // Live-listen so OS-level toggle propagates without a reload.
  function listen() {
    if (typeof window === 'undefined' || !window.matchMedia) return () => {};
    const mq = window.matchMedia(MEDIA_QUERY);
    const fn = (e: MediaQueryListEvent) => {
      prefersReduced = e.matches;
    };
    mq.addEventListener('change', fn);
    return () => mq.removeEventListener('change', fn);
  }

  let cleanup: (() => void) | null = null;

  return {
    get prefersReduced() {
      return prefersReduced;
    },
    get userOverride() {
      return userOverride;
    },
    /** True when the OrbitConcept / OrbitBudgetRing / particle layers
     * may animate. False under OS reduced-motion OR user-override=reduced. */
    get allow() {
      if (userOverride === 'reduced') return false;
      if (userOverride === 'full') return true;
      return !prefersReduced;
    },
    bootstrap() {
      detect();
      if (cleanup) cleanup();
      cleanup = listen() ?? null;
    },
    teardown() {
      if (cleanup) cleanup();
      cleanup = null;
    },
    setUserOverride(mode: 'auto' | 'reduced' | 'full') {
      userOverride = mode;
      // Reflect override on documentElement so the tokens.css attribute
      // selectors fire even on CSS-only consumers.
      if (typeof document !== 'undefined') {
        if (mode === 'auto') document.documentElement.removeAttribute('data-motion');
        else document.documentElement.setAttribute('data-motion', mode);
      }
    },
  };
}

export const motion = createMotion();
