<script lang="ts">
  // OrbitMode1Plan — caption + bubble-up strip + shooting-star overlay for plan mode.
  // Decomposition tree (sun + phase/step/baby) is rendered in 3D by StarCanvas.
  // This component adds: OrbitCaption, BubbleUpStrip, OrbitShootingStarOverlay.

  import type { OrbitFixture, ZoomState, ZoomTarget, ConceptStub } from './types';
  import OrbitCaption from './OrbitCaption.svelte';
  import BubbleUpStrip from './BubbleUpStrip.svelte';
  import OrbitShootingStarOverlay from './OrbitShootingStarOverlay.svelte';

  interface Props {
    fixture: OrbitFixture;
    zoomState: ZoomState;
    zoomToLevel: (target: ZoomTarget) => void;
  }
  let { fixture, zoomState, zoomToLevel: _zoomToLevel }: Props = $props();

  // resolve active concept + active baby for the caption
  const activeConcept = $derived<ConceptStub | undefined>(
    fixture.concepts.find((c) => c.id === fixture.active_concept_id) ?? fixture.concepts[0]
  );

  const activeBabyLabel: string = $derived.by(() => {
    if (!activeConcept) return '';
    const phase = activeConcept.phases.find((p) => p.id === fixture.active_phase_id) ?? activeConcept.phases[0];
    const step = phase?.steps.find((s) => s.id === fixture.active_step_id) ?? phase?.steps[0];
    const baby = step?.baby_steps.find((b) => b.id === fixture.active_baby_step_id) ?? step?.baby_steps[0];
    return baby?.label ?? '';
  });

  // OrbitCaption visibility — gated to the 'concept' zoom level (the
  // far/wide view where the sun IS the focal node). At deeper zooms
  // (phase / step / baby-step), the per-node NodePanel covers the
  // active task with richer HUD content, so the bottom caption would
  // duplicate. Hide it everywhere except concept zoom.
  const showCaption = $derived(zoomState.level === 'concept' && activeBabyLabel.length > 0);
</script>

{#if showCaption}
  <OrbitCaption fixture={fixture} babyLabel={activeBabyLabel} />
{/if}

<!-- Bubble-up event strip: always rendered; shows empty state until B-7 wires SSB -->
<BubbleUpStrip />

<!-- Shooting-star idle effect: fires every ~14s, LCG-seeded from task_id -->
<OrbitShootingStarOverlay task_id={fixture.task_id} />
