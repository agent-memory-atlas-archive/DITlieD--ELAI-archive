<script lang="ts">
  // OrbitDeepSpace — extended starfield outside the main orbit, only
  // visible at zoom <= 1.05. Used in Phase D when research tendrils extend
  // outside the orbit. Phase A renders a placeholder layer that fades in
  // when the orbit zooms out.

  interface Props {
    visible: boolean;
  }
  let { visible }: Props = $props();
</script>

<div class="deep-space" class:visible aria-hidden="true">
  <div class="halo"></div>
</div>

<style>
  .deep-space {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    opacity: 0;
    transition: opacity 320ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  .deep-space.visible { opacity: 0.45; }

  .halo {
    position: absolute;
    inset: -10%;
    background: radial-gradient(ellipse 60% 50% at 50% 50%,
      transparent 30%,
      rgba(30, 138, 122, 0.04) 50%,
      rgba(30, 138, 122, 0.015) 75%,
      transparent 95%);
    filter: blur(2px);
  }

  @media (prefers-reduced-motion: reduce) {
    .deep-space { transition: none; }
  }
</style>
