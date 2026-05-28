<script lang="ts">
  // E-2 Task 2 — SkeletonBubble. Pre-allocates expected response bubble
  // height with a subtle shimmer animation while we wait for TTFT. Visible
  // from submit() until the first streamed token arrives. See
  // workflow/reasoning/E-2-decisions.md DECISION-4 (Bilgihan cite is
  // SUPPORTING ONLY; UX values derived from inline tests + Doherty).
  //
  // Props:
  //   estimatedHeightPx: target height before content arrives. Default 240.
  //   shimmerEnabled: toggle the shimmer animation (off for reduced-motion).

  interface Props {
    estimatedHeightPx?: number;
    shimmerEnabled?: boolean;
  }

  const { estimatedHeightPx = 240, shimmerEnabled = true }: Props = $props();
</script>

<div
  class="skeleton-bubble"
  class:skeleton-bubble--shimmer={shimmerEnabled}
  style="--skel-h: {estimatedHeightPx}px;"
  aria-busy="true"
  aria-label="response is loading"
>
  <div class="skel-line skel-line--w70"></div>
  <div class="skel-line skel-line--w90"></div>
  <div class="skel-line skel-line--w55"></div>
  <div class="skel-line skel-line--w80"></div>
  <div class="skel-line skel-line--w40"></div>
</div>

<style>
  /* Orbit-aesthetic skeleton: faint outline + horizontal lines at varying
     widths. No solid fill (matches orbit-card transparency). */
  .skeleton-bubble {
    position: relative;
    min-height: var(--skel-h, 240px);
    padding: var(--s-4) var(--s-5);
    border-radius: var(--r-md);
    outline: 1px solid var(--border-subtle);
    background: transparent;
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    overflow: hidden;
  }

  .skel-line {
    height: 8px;
    border-radius: var(--r-xs);
    background: var(--border-subtle);
    opacity: 0.6;
  }

  .skel-line--w40 { width: 40%; }
  .skel-line--w55 { width: 55%; }
  .skel-line--w70 { width: 70%; }
  .skel-line--w80 { width: 80%; }
  .skel-line--w90 { width: 90%; }

  /* Shimmer: subtle teal-ghost sweep across the skeleton. Reduced-motion
     consumers pass shimmerEnabled=false. */
  .skeleton-bubble--shimmer::before {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: linear-gradient(
      90deg,
      transparent 0%,
      var(--teal-ghost) 50%,
      transparent 100%
    );
    transform: translateX(-100%);
    animation: skelShimmer 1800ms var(--ease-std) infinite;
  }

  @keyframes skelShimmer {
    from { transform: translateX(-100%); }
    to   { transform: translateX(100%); }
  }

  /* Respect prefers-reduced-motion at the OS level even when shimmerEnabled
     is true. The shimmer pseudo-element collapses to opacity:0. */
  @media (prefers-reduced-motion: reduce) {
    .skeleton-bubble--shimmer::before {
      animation: none;
      opacity: 0;
    }
  }
</style>
