<script lang="ts">
  // E-2 Task 2 — CursorSentinel. CSS-only blinking caret rendered next to
  // a streaming response. The blink is driven by a CSS keyframe (no JS
  // interval) so the cursor stays smooth even when the main thread is
  // busy with FramePacer drains. Disappears on stream_complete.

  interface Props {
    visible?: boolean;
  }

  const { visible = true }: Props = $props();
</script>

{#if visible}
  <span class="cursor-sentinel" aria-hidden="true"></span>
{/if}

<style>
  .cursor-sentinel {
    display: inline-block;
    width: 6px;
    height: 1.1em;
    margin-left: 1px;
    vertical-align: text-bottom;
    background: var(--teal);
    animation: cursorBlink 1100ms steps(1, end) infinite;
  }

  @keyframes cursorBlink {
    0%, 49%   { opacity: 1; }
    50%, 100% { opacity: 0; }
  }

  @media (prefers-reduced-motion: reduce) {
    .cursor-sentinel {
      animation: none;
      opacity: 0.85;
    }
  }
</style>
