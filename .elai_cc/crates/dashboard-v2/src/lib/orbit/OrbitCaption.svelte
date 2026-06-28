<script lang="ts">
  // OrbitCaption — plain-language caption rendered below the orbit.
  // Pure deterministic mapping via captionFor(); no LLM, no randomness.
  // Same task + same FSM/GRC/intent => identical caption.

  import { captionFor } from './captions';
  import type { OrbitFixture } from './types';

  interface Props {
    fixture: OrbitFixture;
    babyLabel: string;
  }
  let { fixture, babyLabel }: Props = $props();

  const text = $derived(captionFor({
    fsm: fixture.fsm,
    grc_step: fixture.grc_step ?? 'status',
    baby_step_label: babyLabel,
    bubble_action: fixture.bubble_action,
    research_topic: fixture.idea,
  }));
</script>

<div class="caption" aria-live="polite">
  <div class="prefix">elai</div>
  <div class="text">{text}</div>
  <div class="line"></div>
</div>

<style>
  .caption {
    position: absolute;
    bottom: var(--s-5);
    left: 50%;
    transform: translateX(-50%);
    z-index: 3;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    pointer-events: none;
    max-width: 70%;
  }
  .prefix {
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--teal);
    opacity: 0.85;
  }
  .text {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: var(--leading-code);
    color: var(--text-primary);
    text-align: center;
  }
  .line {
    height: 1px;
    background: var(--teal);
    width: 0;
    animation: grow-line 1.4s ease-out forwards;
    max-width: 200px;
    align-self: center;
  }
  @keyframes grow-line {
    from { width: 0;   opacity: 0.7; }
    to   { width: 60px; opacity: 0.5; }
  }
  @media (prefers-reduced-motion: reduce) {
    .line { animation: none; width: 60px; opacity: 0.5; }
  }
</style>
