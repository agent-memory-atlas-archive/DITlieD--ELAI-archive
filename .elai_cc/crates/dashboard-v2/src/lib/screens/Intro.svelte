<script lang="ts">
  // ============================================================
  // Intro.svelte — the cosmic brand entrance (DESIGN.md §5 + §13).
  //
  // Staged cinematic build from pure black: starfield (3-layer
  // parallax canvas) -> teal bloom (always on) -> three hairline
  // rings -> white hairline stroke-draw wordmark (§5 geometry) ->
  // START -> exit curtain dissolve to the selector's exact bg.
  // The sun/moon toggle drives the theme store. The LOCAL/CLOUD/HYBRID
  // modes now live on the selector, not here. NO TAGLINE (user
  // decision — the logo is the sole focal element).
  //
  // Port of workflow/design/mockup/intro.html, reimplemented as a
  // real Svelte 5 component (runes). Fully reduced-motion aware:
  // every animation resolves to its static end state.
  // ============================================================
  import { onMount } from 'svelte';
  import { session } from '../state/session.svelte';
  import { theme } from '../state/theme.svelte';
  import TopRightControls from '../chrome/TopRightControls.svelte';

  let leaving = $state<boolean>(false); // exit curtain raised

  let sky = $state<HTMLCanvasElement | null>(null);
  let logoSvg = $state<SVGSVGElement | null>(null);

  function reducedMotion(): boolean {
    return (
      typeof window !== 'undefined' &&
      window.matchMedia('(prefers-reduced-motion:reduce)').matches
    );
  }

  function replayLogo() {
    if (!logoSvg || reducedMotion()) return;
    for (const ln of Array.from(logoSvg.querySelectorAll('line'))) {
      const el = ln as SVGLineElement;
      el.style.animation = 'none';
      // force reflow so the animation restarts
      void el.getBoundingClientRect();
      el.style.animation = '';
    }
  }

  function start() {
    if (leaving) return;
    leaving = true;
    const go = () => session.enterSelector();
    if (reducedMotion()) {
      go();
    } else {
      // match the .curtain dissolve duration (500ms)
      window.setTimeout(go, 500);
    }
  }

  // ── 3-layer parallax starfield (§13 step 1). Every star is a pure
  //    function of a SHARED wall clock + per-star hash so the field is
  //    continuous across the intro->selector hop with no blink. Each
  //    star fades up / holds / fades out (sin life) then is reborn at a
  //    fresh hashed position at alpha~0 (relocation never visible).
  onMount(() => {
    if (!sky) return;
    const canvas: HTMLCanvasElement = sky;
    const ctx2d = canvas.getContext('2d');
    if (!ctx2d) return;
    const ctx: CanvasRenderingContext2D = ctx2d;
    const root = document.documentElement;
    const reduce = reducedMotion();
    const ENTRANCE = 1600; // ms fade-up of the whole field
    const dpr = Math.min(window.devicePixelRatio || 1, 2);

    // far / mid / near: radius range, max alpha, life-period range (ms)
    const LAYERS = [
      { rMin: 0.35, rMax: 0.85, aMax: 0.55, pMin: 2600, pMax: 6200 },
      { rMin: 0.8, rMax: 1.55, aMax: 0.74, pMin: 3600, pMax: 8200 },
      { rMin: 1.7, rMax: 2.9, aMax: 0.96, pMin: 4800, pMax: 11000 },
    ];

    type Star = { i: number; li: number; p: number; off: number };
    let stars: Star[] = [];
    let W = 0;
    let H = 0;
    let start: number | null = null;
    let raf = 0;
    let resizeTimer: number | null = null;

    // deterministic hash -> [0,1): identical on both screens for (a,b)
    function hash(a: number, b: number): number {
      let h = (a ^ 0x9e3779b9) >>> 0;
      h = Math.imul(h ^ ((b + 0x85ebca6b) | 0), 0xc2b2ae35);
      h ^= h >>> 15;
      h = Math.imul(h, 0x27d4eb2f);
      h ^= h >>> 13;
      return (h >>> 0) / 4294967296;
    }

    function build() {
      W = canvas.clientWidth;
      H = canvas.clientHeight;
      canvas.width = Math.round(W * dpr);
      canvas.height = Math.round(H * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const scale = (W * H) / (1920 * 960);
      const counts = [
        Math.round(9 * scale) + 2,
        Math.round(5 * scale) + 2,
        Math.round(2 * scale) + 1,
      ];
      stars = [];
      let idx = 0;
      LAYERS.forEach((L, li) => {
        for (let i = 0; i < counts[li]; i++) {
          const p = L.pMin + hash(idx * 131 + 7, 1) * (L.pMax - L.pMin);
          stars.push({ i: idx, li, p, off: hash(idx * 131 + 19, 2) * p });
          idx++;
        }
      });
    }

    function paint(entrance: number) {
      const light = root.getAttribute('data-theme') === 'light';
      const col = light ? '12,16,15' : '255,255,255';
      ctx.clearRect(0, 0, W, H);
      ctx.fillStyle = 'rgb(' + col + ')';
      const now = Date.now();
      for (const s of stars) {
        const L = LAYERS[s.li];
        const tn = now + s.off;
        const c = Math.floor(tn / s.p);
        const lt = tn - c * s.p;
        const life = reduce ? 0.7 : Math.sin((Math.PI * lt) / s.p);
        if (life <= 0) continue;
        const x = hash(s.i * 8, c) * W;
        const y = hash(s.i * 8 + 1, c) * H;
        const r = L.rMin + hash(s.i * 8 + 2, c) * (L.rMax - L.rMin);
        const am = L.aMax * (0.55 + 0.45 * hash(s.i * 8 + 3, c));
        const a = am * life * entrance;
        if (a < 0.012) continue;
        ctx.globalAlpha = a;
        ctx.beginPath();
        ctx.arc(x, y, r, 0, 6.2832);
        ctx.fill();
      }
      ctx.globalAlpha = 1;
    }

    function loop(ts: number) {
      if (start === null) start = ts;
      const entrance = ENTRANCE <= 0 ? 1 : Math.min(1, (ts - start) / ENTRANCE);
      paint(entrance);
      raf = requestAnimationFrame(loop);
    }

    function init() {
      build();
      if (reduce) {
        paint(1); // static held frame
      } else {
        cancelAnimationFrame(raf);
        start = null;
        raf = requestAnimationFrame(loop);
      }
    }

    function onResize() {
      if (resizeTimer !== null) window.clearTimeout(resizeTimer);
      resizeTimer = window.setTimeout(init, 180);
    }
    // Repaint a held frame when the theme flips under reduced motion
    // (the animation loop handles the live case automatically).
    function onThemeChange() {
      if (reduce) paint(1);
    }

    window.addEventListener('resize', onResize);
    document.addEventListener('elai-themechange', onThemeChange);
    init();

    return () => {
      cancelAnimationFrame(raf);
      if (resizeTimer !== null) window.clearTimeout(resizeTimer);
      window.removeEventListener('resize', onResize);
      document.removeEventListener('elai-themechange', onThemeChange);
    };
  });

  // Notify the starfield loop on theme change (it samples data-theme
  // each paint; under reduced motion it needs an explicit nudge).
  $effect(() => {
    // read theme.current so this effect re-runs on toggle
    void theme.current;
    if (typeof document !== 'undefined') {
      document.dispatchEvent(new Event('elai-themechange'));
    }
  });
</script>

<main class="stage" aria-label="ELAI — welcome">
  <div class="bloom" aria-hidden="true"></div>
  <canvas class="starfield" bind:this={sky} aria-hidden="true"></canvas>

  <!-- thin full-width drag strip so the frameless cosmic screen is
       movable; sits above the cosmic visuals but BELOW every
       interactive control (toggles/logo/modes/START). Double-click =
       toggle-maximize (Tauri drag-region default). -->
  <div class="dragstrip" data-tauri-drag-region aria-hidden="true"></div>

  <div class="tr-cluster">
    <TopRightControls />
  </div>

  <div class="system">
    <div class="ring r1"></div>
    <div class="ring r2"></div>
    <div class="ring r3"></div>

    <div class="core">
      <button
        class="logo-wrap"
        type="button"
        title="ELAI — click to replay"
        aria-label="ELAI"
        onclick={replayLogo}
      >
        <svg class="logo" bind:this={logoSvg} viewBox="-3 -3 236 66" role="img" aria-label="ELAI logo">
          <line pathLength="1" x1="0" y1="0" x2="0" y2="60" /><line pathLength="1" x1="0" y1="0" x2="34" y2="0" />
          <line pathLength="1" x1="0" y1="30" x2="25" y2="30" /><line pathLength="1" x1="0" y1="60" x2="34" y2="60" />
          <line pathLength="1" x1="76" y1="0" x2="76" y2="60" /><line pathLength="1" x1="76" y1="60" x2="106" y2="60" />
          <line pathLength="1" x1="150" y1="60" x2="170" y2="0" /><line pathLength="1" x1="170" y1="0" x2="190" y2="60" />
          <line pathLength="1" x1="162.7" y1="38" x2="177.3" y2="38" /><line pathLength="1" x1="228" y1="0" x2="228" y2="60" />
        </svg>
      </button>
    </div>
  </div>

  <div class="readout">
    <button class="np" type="button" onclick={start}>START</button>
  </div>

  <div class="curtain" class:show={leaving} aria-hidden="true"></div>
</main>

<style>
  /* §13 entrance — BRAND tier; the void cosmos atmosphere lives ONLY
     here. The dark vignette over --void is the "black" we fade up from. */
  .stage {
    position: fixed;
    inset: 0;
    overflow: hidden;
    font-family: var(--font-disp);
    color: var(--text);
    background:
      radial-gradient(
        ellipse 135% 135% at 50% 50%,
        rgba(4, 13, 17, 1) 0%,
        rgba(4, 13, 17, 0.92) 18%,
        rgba(4, 13, 17, 0.74) 36%,
        rgba(4, 13, 17, 0.52) 54%,
        rgba(4, 13, 17, 0.28) 72%,
        rgba(4, 13, 17, 0.08) 90%,
        rgba(4, 13, 17, 0) 100%
      ),
      var(--void);
  }
  :global(:root[data-theme='light']) .stage {
    background:
      radial-gradient(ellipse 135% 135% at 50% 50%, #eef2ef 0%, #e8ede9 40%, #d6ddd9 80%, #cad2cd 100%),
      var(--void);
  }

  .starfield {
    position: absolute;
    inset: 0;
    z-index: 1;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  /* teal bloom on its own layer over the stars + vignette. Always on
     (the NO GLOW toggle was removed). §13 step 2. */
  .bloom {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    opacity: 0;
    background: radial-gradient(
      ellipse 54% 54% at 50% 50%,
      rgba(43, 187, 164, 0.16) 0%,
      rgba(43, 187, 164, 0.06) 30%,
      rgba(43, 187, 164, 0.018) 50%,
      rgba(43, 187, 164, 0) 68%
    );
    animation: introFade 1500ms var(--ease-out) 600ms forwards;
  }
  :global(:root[data-theme='light']) .bloom {
    background: radial-gradient(
      ellipse 56% 56% at 50% 50%,
      rgba(43, 187, 164, 0.22) 0%,
      rgba(43, 187, 164, 0.09) 30%,
      rgba(43, 187, 164, 0.025) 50%,
      rgba(43, 187, 164, 0) 68%
    );
  }
  /* orbital rings — fade in staggered (§13 step 3) */
  .system {
    position: absolute;
    inset: 0;
    z-index: 2;
    pointer-events: none;
  }
  .ring {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    border: 1px solid var(--ring-teal);
    border-radius: 50%;
    opacity: 0;
    animation: introFade 1400ms var(--ease-out) forwards;
  }
  .ring.r1 {
    width: 370px;
    height: 370px;
    animation-delay: 500ms;
  }
  .ring.r2 {
    width: 560px;
    height: 560px;
    animation-delay: 700ms;
  }
  .ring.r3 {
    width: 750px;
    height: 750px;
    border-color: var(--ring-teal-faint);
    animation-delay: 900ms;
  }

  .core {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    z-index: 3;
    display: flex;
    flex-direction: column;
    align-items: center;
    pointer-events: none;
  }
  .logo-wrap {
    appearance: none;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    pointer-events: auto;
    line-height: 0;
    filter: drop-shadow(0 0 42px rgba(43, 187, 164, 0.36))
      drop-shadow(0 0 12px rgba(43, 187, 164, 0.26));
  }
  .logo {
    width: 300px;
    height: auto;
    display: block;
  }
  /* §5: each stroke draws via stroke-dashoffset; first at 900ms, +340ms
     per stroke, last finishes ~5360ms — staged after the scene fades up. */
  .logo line {
    fill: none;
    stroke: var(--text);
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-dasharray: 1;
    stroke-dashoffset: 1;
    animation: introDraw 1400ms var(--ease-draw) forwards;
  }
  .logo line:nth-child(1) { animation-delay: 900ms; }
  .logo line:nth-child(2) { animation-delay: 1240ms; }
  .logo line:nth-child(3) { animation-delay: 1580ms; }
  .logo line:nth-child(4) { animation-delay: 1920ms; }
  .logo line:nth-child(5) { animation-delay: 2260ms; }
  .logo line:nth-child(6) { animation-delay: 2600ms; }
  .logo line:nth-child(7) { animation-delay: 2940ms; }
  .logo line:nth-child(8) { animation-delay: 3280ms; }
  .logo line:nth-child(9) { animation-delay: 3620ms; }
  .logo line:nth-child(10) { animation-delay: 3960ms; }

  /* START — fades in after the logo lands (§13 step 6) */
  .readout {
    position: fixed;
    left: 50%;
    bottom: 44px;
    transform: translateX(-50%);
    z-index: 5;
    display: flex;
    justify-content: center;
    opacity: 0;
    animation: introFade 600ms var(--ease-out) 5200ms forwards;
  }
  .np {
    position: relative;
    appearance: none;
    cursor: pointer;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 0;
    font-family: var(--font-disp);
    font-weight: 500;
    font-size: 12px;
    letter-spacing: 0.2em;
    color: var(--read);
    padding: 13px 34px;
    overflow: hidden;
    transition:
      color var(--dur-fast) var(--ease-out),
      border-color var(--dur-fast) var(--ease-out);
  }
  .np:hover {
    color: var(--d-teal-bright);
    border-color: var(--d-teal-soft);
  }
  .np::after {
    content: '';
    position: absolute;
    left: 0;
    bottom: 0;
    height: 1px;
    width: 100%;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .np:hover::after {
    transform: scaleX(1);
  }

  /* thin full-width top drag strip — above cosmic visuals (starfield
     z1 / bloom z0), below every interactive control. Transparent. */
  .dragstrip {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 34px;
    z-index: 4;
    background: transparent;
  }

  /* top-right control cluster — IDENTICAL component, size and position to
     the selector + workflow shell (right:6px, vertically centered in the
     48px top zone) so the 4 buttons never pop between screens. Only the
     scene fade-in is intro-specific. */
  .tr-cluster {
    position: fixed;
    top: 18px;
    right: 20px;
    z-index: 7;
    display: flex;
    align-items: center;
    opacity: 0;
    animation: introFade 800ms var(--ease-out) 250ms forwards;
  }

  :global(:root[data-theme='light']) .np::after {
    background: var(--d-teal-deep);
  }
  :global(:root[data-theme='light']) .np:hover {
    color: var(--d-teal-deep);
  }

  /* exit curtain — dissolves to the selector's exact bg (#0a0f0e =
     --bg) so the screen hop has no flash. §13 step 6. */
  .curtain {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: var(--bg);
    opacity: 0;
    pointer-events: none;
    transition: opacity 500ms var(--ease-out);
  }
  .curtain.show {
    opacity: 1;
    pointer-events: auto;
  }

  @keyframes introFade {
    to {
      opacity: 1;
    }
  }
  @keyframes introDraw {
    to {
      stroke-dashoffset: 0;
    }
  }

  /* §7 + §13: reduced-motion resolves the whole sequence to its static
     end state (logo drawn, scene present, curtain instant). The
     starfield JS already paints a held frame. */
  @media (prefers-reduced-motion: reduce) {
    .logo line {
      animation: none;
      stroke-dashoffset: 0;
    }
    .bloom,
    .ring,
    .tr-cluster,
    .readout {
      animation: none;
      opacity: 1;
    }
    .curtain {
      transition: none;
    }
  }
</style>
