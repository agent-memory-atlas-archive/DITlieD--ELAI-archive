<script lang="ts">
  // OrbitConcept — renders a single concept as a decomposition starmap
  // anchored at (cx, cy). Active concepts show full phase/step/baby
  // rings; inactive concepts render a compressed marker (dot + tiny
  // phase ring + label) so the user can see neighbouring constellations
  // without overwhelming the canvas.

  import type {
    ConceptStub, OrbitFixture, ZoomState, ZoomTarget, PhaseStub, StepStub, BabyStepStub,
  } from './types';
  import { polar, curve } from './geometry';
  import { lcg, hashId, combineSeeds } from './lcg';
  import OrbitGrcDots from './OrbitGrcDots.svelte';
  import OrbitBudgetRing from './OrbitBudgetRing.svelte';
  import { motion } from '../state/motion.svelte';

  interface Props {
    concept: ConceptStub;
    cx: number;
    cy: number;
    isActive: boolean;
    fixture: OrbitFixture;
    zoomState: ZoomState;
    zoomToLevel: (target: ZoomTarget) => void;
    onSelectConcept: (conceptId: string) => void;
  }
  let { concept, cx, cy, isActive, fixture, zoomState, zoomToLevel, onSelectConcept }: Props = $props();

  // E-33: reduced-motion guard for SVG <animate> elements. CSS @media
  // (prefers-reduced-motion) at the bottom of this file already kills
  // CSS-class animations; the JS guard here covers <animate> declarative
  // SVG animation elements which the CSS media query does not reach.
  let allowMotion = $derived(motion.allow);

  // ring radii (slightly tighter than the trial when multi-concept,
  // unchanged for single-concept layouts)
  const R_PHASE = 95;
  const R_STEP  = 162;
  const R_BABY  = 226;

  const phases    = $derived<PhaseStub[]>(concept.phases);
  const activePhase = $derived<PhaseStub | undefined>(
    isActive ? (phases.find((p) => p.id === fixture.active_phase_id) ?? phases[0]) : undefined
  );
  const steps     = $derived<StepStub[]>(activePhase?.steps ?? []);
  const activeStep = $derived<StepStub | undefined>(
    isActive ? (steps.find((s) => s.id === fixture.active_step_id) ?? steps[0]) : undefined
  );
  const babySteps = $derived<BabyStepStub[]>(activeStep?.baby_steps ?? []);
  const activeBaby = $derived<BabyStepStub | undefined>(
    isActive ? (babySteps.find((b) => b.id === fixture.active_baby_step_id) ?? babySteps[0]) : undefined
  );

  // -----------------------------------------------------------------
  // angles + deterministic jitter (per concept)
  // -----------------------------------------------------------------
  const seedRoot = $derived(combineSeeds(hashId(fixture.task_id), hashId(concept.id)));

  function phaseAngle(i: number, n: number): number {
    return -Math.PI / 2 + (2 * Math.PI * i) / Math.max(n, 1);
  }
  function activePhaseIdx(): number { return phases.findIndex((p) => p.id === activePhase?.id); }
  function activeStepIdx(): number { return steps.findIndex((s) => s.id === activeStep?.id); }

  // Each phase claims an angular footprint of 2π/phaseCount. Steps spread
  // within ~70% of that footprint centred on their parent phase's angle.
  // Babies spread within ~60% of their parent step's footprint.
  function stepAngleFor(parentPhaseIdx: number, parentPhaseCount: number, stepIdx: number, stepCount: number): number {
    const baseTheta = phaseAngle(parentPhaseIdx, parentPhaseCount);
    if (stepCount <= 1) return baseTheta;
    const phaseFootprint = (2 * Math.PI) / Math.max(parentPhaseCount, 1);
    const arc = phaseFootprint * 0.70;
    const offset = (stepIdx - (stepCount - 1) / 2) * (arc / Math.max(stepCount - 1, 1));
    return baseTheta + offset;
  }
  function babyAngleFor(parentPhaseIdx: number, parentPhaseCount: number, parentStepIdx: number, parentStepCount: number, babyIdx: number, babyCount: number): number {
    const stepTheta = stepAngleFor(parentPhaseIdx, parentPhaseCount, parentStepIdx, parentStepCount);
    if (babyCount <= 1) return stepTheta;
    const phaseFootprint = (2 * Math.PI) / Math.max(parentPhaseCount, 1);
    const stepFootprint = parentStepCount <= 1 ? phaseFootprint * 0.70 : phaseFootprint * 0.70 / parentStepCount;
    const arc = stepFootprint * 0.85;
    const offset = (babyIdx - (babyCount - 1) / 2) * (arc / Math.max(babyCount - 1, 1));
    return stepTheta + offset;
  }
  // Backwards-compat aliases (used by active-branch rendering below).
  function stepAngle(i: number, n: number): number {
    return stepAngleFor(activePhaseIdx(), phases.length, i, n);
  }
  function babyAngle(i: number, n: number): number {
    return babyAngleFor(activePhaseIdx(), phases.length, activeStepIdx(), steps.length, i, n);
  }
  // Two seeded streams per id so radius + angle jitter don't correlate.
  function jitterPair(id: string): { jr: number; jtheta: number } {
    const rng = lcg(combineSeeds(seedRoot, hashId(id)));
    const jr = (rng() - 0.5) * 2;       // [-1, 1]
    const jtheta = (rng() - 0.5) * 2;   // [-1, 1]
    return { jr, jtheta };
  }
  function pos(theta: number, r: number, id: string) {
    const { jr, jtheta } = jitterPair(id);
    // Generous jitter — nodes break the ring so the layout looks like a
    // constellation rather than a perfect concentric circle.
    const rj = r + jr * 14;
    const tj = theta + jtheta * 0.07;
    return polar(tj, rj, cx, cy);
  }
  // Edge bow factor varies per edge id-pair so curves don't all look identical.
  function bowFor(idA: string, idB: string, base = 0.16): number {
    const rng = lcg(combineSeeds(hashId(idA), hashId(idB)));
    return base + (rng() - 0.5) * 0.10;
  }

  // -----------------------------------------------------------------
  // status + visual mapping
  // -----------------------------------------------------------------
  type NodeStatus = 'active' | 'done' | 'pending';
  function phaseStatus(p: PhaseStub): NodeStatus {
    if (!isActive) return 'pending';
    if (p.id === fixture.active_phase_id) return 'active';
    const ai = phases.findIndex((x) => x.id === fixture.active_phase_id);
    const i = phases.findIndex((x) => x.id === p.id);
    if (ai < 0) return 'pending';
    return i < ai ? 'done' : 'pending';
  }
  function stepStatus(s: StepStub): NodeStatus {
    if (s.id === fixture.active_step_id) return 'active';
    const ai = steps.findIndex((x) => x.id === fixture.active_step_id);
    const i = steps.findIndex((x) => x.id === s.id);
    if (ai < 0) return 'pending';
    return i < ai ? 'done' : 'pending';
  }
  function babyStatus(b: BabyStepStub): NodeStatus {
    if (b.id === fixture.active_baby_step_id) return 'active';
    const ai = babySteps.findIndex((x) => x.id === fixture.active_baby_step_id);
    const i = babySteps.findIndex((x) => x.id === b.id);
    if (ai < 0) return 'pending';
    return i < ai ? 'done' : 'pending';
  }
  function colorFor(status: NodeStatus): string {
    if (status === 'active') return 'var(--teal-bright)';
    if (status === 'done')   return 'var(--teal)';
    return 'var(--text-disabled)';
  }
  function radiusFor(status: NodeStatus, kind: 'phase' | 'step' | 'baby'): number {
    const base = kind === 'phase' ? 6 : kind === 'step' ? 4.5 : 5;
    if (status === 'active') return base + 1;
    if (status === 'done')   return base * 0.7;
    return base * 0.55;
  }

  const FSM_TOKEN: Readonly<Record<string, string>> = {
    IDLE: 'var(--state-idle)', PLANNING: 'var(--state-planning)',
    FEASIBILITY_PROBE: 'var(--state-feasibility)', TESTING_FIRST: 'var(--state-testing)',
    EXECUTING: 'var(--state-executing)', AUDITING: 'var(--state-auditing)',
    MEMORIZING: 'var(--state-memorizing)', COMPLETE: 'var(--state-complete)',
    CONFIGURING: 'var(--state-configuring)', RESEARCHING: 'var(--state-researching)',
  };
  const haloColor = $derived(isActive ? (FSM_TOKEN[fixture.fsm] ?? 'var(--state-idle)') : 'var(--text-ghost)');

  let hovered = $state<string | null>(null);
  function clickPhase(p: PhaseStub, x: number, y: number) {
    zoomToLevel({ level: 'phase', node_id: p.id, x, y, scale: 1.6 });
  }
  function clickStep(s: StepStub, x: number, y: number) {
    zoomToLevel({ level: 'step', node_id: s.id, x, y, scale: 2.4 });
  }
  function clickBaby(b: BabyStepStub, x: number, y: number) {
    zoomToLevel({ level: 'baby-step', node_id: b.id, x, y, scale: 3.4 });
  }
  function clickConceptCenter() {
    if (!isActive) onSelectConcept(concept.id);
    else zoomToLevel({ level: 'concept', node_id: concept.id, x: cx, y: cy, scale: 1.0 });
  }

  // active baby world position — used to anchor GRC dots + caption.
  const activeBabyPos: { x: number; y: number } | null = $derived.by(() => {
    if (!isActive || !activeBaby) return null;
    const i = babySteps.findIndex((b) => b.id === activeBaby.id);
    if (i < 0) return null;
    return pos(babyAngle(i, babySteps.length), R_BABY, activeBaby.id);
  });

  // expose position so the parent can place a caption above it
  const _exposedActiveBaby = $derived(activeBabyPos);
  // export note: no actual export needed since orbit caption is rendered
  // by OrbitMode1Plan once it knows the active concept.
</script>

<g class="concept-group" class:concept-group--inactive={!isActive}
   transform-origin="{cx} {cy}">
  <!-- All 2D-overlay markers removed — the 3D galaxy is the canonical
       view. Old inactive-concept dashed ring + evidence-gate pips +
       constellation-breathe animation are all gone (they sat in 2D
       screen space and didn't fit the depth illusion, plus 5+
       simultaneous SVG transform animations were taxing FPS). -->
  {#if false}{/if}

  <!-- ACTIVE concept content. Hidden via CSS (.svg-orbit-content
       { display: none; }) — the three.js scene now owns the orbit
       rendering. SVG stayed in 2D screen-space and broke the
       "looking-down-at-disc" depth illusion. Toggle the class for
       debugging. -->
  {#if isActive && activePhase}
    <g class="svg-orbit-content">
    <!-- legacy SVG decomposition tree below -->
    {#if true}
    {@const ap = pos(phaseAngle(activePhaseIdx(), phases.length), R_PHASE, activePhase.id)}

    <!-- BACKDROP: dim dots + faint hint-lines for EVERY step + baby across
         ALL phases. This is what gives the constellation feel — points all
         around the orbit, not just on the active branch. -->
    <!-- spokes from centre to every phase, very faint -->
    {#each phases as bp_phase, pi (bp_phase.id + '-bd-spoke')}
      {@const bpx = pos(phaseAngle(pi, phases.length), R_PHASE, bp_phase.id)}
      {#if bp_phase.id !== activePhase.id}
        <path d={curve(cx, cy, bpx.x, bpx.y, bowFor(concept.id, bp_phase.id, 0.16))}
              fill="none" stroke="var(--border-subtle)" stroke-width="0.5" opacity="0.45" />
      {/if}
    {/each}

    {#each phases as bp_phase, pi (bp_phase.id + '-bd-phase')}
      {@const bpx = pos(phaseAngle(pi, phases.length), R_PHASE, bp_phase.id)}
      {#each bp_phase.steps as bp_step, si (bp_step.id + '-bd-step')}
        {@const bps = pos(stepAngleFor(pi, phases.length, si, bp_phase.steps.length), R_STEP, bp_step.id)}
        {#if !(bp_phase.id === activePhase.id && bp_step.id === activeStep?.id)}
          <circle cx={bps.x} cy={bps.y} r="2.4" fill="var(--steel)" opacity="0.6" />
        {/if}
        {#each bp_step.baby_steps as bp_baby, bi (bp_baby.id + '-bd-baby')}
          {@const bpb = pos(babyAngleFor(pi, phases.length, si, bp_phase.steps.length, bi, bp_step.baby_steps.length), R_BABY, bp_baby.id)}
          {#if !(bp_phase.id === activePhase.id && bp_step.id === activeStep?.id)}
            <circle cx={bpb.x} cy={bpb.y} r="1.6" fill="var(--text-secondary)" opacity="0.45" />
          {/if}
        {/each}
        <!-- Hint line from phase to its steps (faint) -->
        {#if bp_phase.id !== activePhase.id}
          <path d={curve(bpx.x, bpx.y, bps.x, bps.y, bowFor(bp_phase.id, bp_step.id, 0.12))}
                fill="none" stroke="var(--border-default)" stroke-width="0.6" opacity="0.5" />
        {/if}
      {/each}
    {/each}

    <!-- inactive phase->step lines (within the ACTIVE phase: faint pulse-flow) -->
    {#each steps as s, i (s.id)}
      {@const sp = pos(stepAngle(i, steps.length), R_STEP, s.id)}
      {#if s.id !== activeStep?.id}
        <path d={curve(ap.x, ap.y, sp.x, sp.y, bowFor(activePhase.id, s.id, 0.14))}
              fill="none" stroke="var(--border-default)" stroke-width="0.8" opacity="0.55" class="ambient-line" />
      {/if}
    {/each}

    {#if activeStep}
      {@const as2 = pos(stepAngle(activeStepIdx(), steps.length), R_STEP, activeStep.id)}
      {#each babySteps as b, i (b.id)}
        {@const bp = pos(babyAngle(i, babySteps.length), R_BABY, b.id)}
        {#if b.id !== activeBaby?.id}
          <path d={curve(as2.x, as2.y, bp.x, bp.y, bowFor(activeStep.id, b.id, 0.12))}
                fill="none" stroke="var(--border-default)" stroke-width="0.7" opacity="0.45" class="ambient-line" />
        {/if}
      {/each}
    {/if}

    <!-- active path -->
    <path d={curve(cx, cy, ap.x, ap.y, bowFor(concept.id, activePhase.id, 0.20))} fill="none" stroke="var(--teal-line)" stroke-width="1.5" />
    {#if activeStep}
      {@const as2 = pos(stepAngle(activeStepIdx(), steps.length), R_STEP, activeStep.id)}
      <path d={curve(ap.x, ap.y, as2.x, as2.y, bowFor(activePhase.id, activeStep.id, 0.18))} fill="none" stroke="var(--teal-line)" stroke-width="1.5" />
      {#if activeBaby}
        {@const aIdx = babySteps.findIndex((bx) => bx.id === activeBaby.id)}
        {@const ab = pos(babyAngle(aIdx, babySteps.length), R_BABY, activeBaby.id)}
        <path d={curve(as2.x, as2.y, ab.x, ab.y, bowFor(activeStep.id, activeBaby.id, 0.16))} fill="none" stroke="var(--teal)" stroke-width="2" class="active-line" />
      {/if}
    {/if}

    <!-- phase nodes -->
    {#each phases as p, i (p.id)}
      {@const xy = pos(phaseAngle(i, phases.length), R_PHASE, p.id)}
      {@const status = phaseStatus(p)}
      {@const r = radiusFor(status, 'phase')}
      {@const isPhaseActive = status === 'active'}
      <g class="node" role="button" tabindex="0"
         onclick={() => clickPhase(p, xy.x, xy.y)}
         onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') clickPhase(p, xy.x, xy.y); }}
         onmouseenter={() => { hovered = p.id; }} onmouseleave={() => { hovered = null; }}
         aria-label="phase: {p.label}">
        {#if isPhaseActive && allowMotion}
          <circle cx={xy.x} cy={xy.y} r={r + 6} fill="none" stroke="var(--teal-bright)" stroke-width="1" opacity="0.35">
            <animate attributeName="r" values="{r + 4};{r + 13};{r + 4}" dur="3s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.35;0;0.35" dur="3s" repeatCount="indefinite" />
          </circle>
        {/if}
        <circle cx={xy.x} cy={xy.y} r={r} fill={colorFor(status)} filter={isPhaseActive ? 'url(#bloom-soft)' : undefined}>
          {#if isPhaseActive && allowMotion}
            <animate attributeName="opacity" values="0.6;1;0.6" dur="2.4s" repeatCount="indefinite" />
          {/if}
        </circle>
        <!-- always-visible faint phase label (like v14); active gets bright -->
        <text x={xy.x + (xy.x > cx + 4 ? 14 : xy.x < cx - 4 ? -14 : 0)}
              y={xy.y + (xy.y > cy + 4 ? 20 : -12)}
              text-anchor={xy.x > cx + 4 ? 'start' : xy.x < cx - 4 ? 'end' : 'middle'}
              class="label label--phase" class:label--active={isPhaseActive} class:label--ambient={!isPhaseActive && hovered !== p.id}
              paint-order="stroke" stroke="var(--bg-base)" stroke-width="3" stroke-linejoin="round">
          {p.label.toLowerCase()}
        </text>
      </g>
    {/each}

    <!-- step nodes -->
    {#each steps as s, i (s.id)}
      {@const xy = pos(stepAngle(i, steps.length), R_STEP, s.id)}
      {@const status = stepStatus(s)}
      {@const r = radiusFor(status, 'step')}
      {@const isStepActive = status === 'active'}
      <g class="node" role="button" tabindex="0"
         onclick={() => clickStep(s, xy.x, xy.y)}
         onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') clickStep(s, xy.x, xy.y); }}
         onmouseenter={() => { hovered = s.id; }} onmouseleave={() => { hovered = null; }}
         aria-label="step: {s.label}">
        {#if isStepActive && allowMotion}
          <circle cx={xy.x} cy={xy.y} r={r + 5} fill="none" stroke="var(--teal-bright)" stroke-width="1" opacity="0.3">
            <animate attributeName="r" values="{r + 3};{r + 10};{r + 3}" dur="2.8s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.3;0;0.3" dur="2.8s" repeatCount="indefinite" />
          </circle>
        {/if}
        <circle cx={xy.x} cy={xy.y} r={r} fill={colorFor(status)} filter={isStepActive ? 'url(#bloom-soft)' : undefined}>
          {#if isStepActive && allowMotion}
            <animate attributeName="opacity" values="0.6;1;0.6" dur="2.4s" repeatCount="indefinite" />
          {/if}
        </circle>
        {#if isStepActive || hovered === s.id}
          <text x={xy.x + (xy.x > cx + 4 ? 10 : -10)} y={xy.y + 4}
                text-anchor={xy.x > cx + 4 ? 'start' : 'end'}
                class="label label--step" class:label--active={isStepActive}
                paint-order="stroke" stroke="var(--bg-base)" stroke-width="2.5" stroke-linejoin="round">
            {s.label.toLowerCase()}
          </text>
        {/if}
      </g>
    {/each}

    <!-- baby nodes -->
    {#each babySteps as b, i (b.id)}
      {@const xy = pos(babyAngle(i, babySteps.length), R_BABY, b.id)}
      {@const status = babyStatus(b)}
      {@const r = radiusFor(status, 'baby')}
      {@const isBabyActive = status === 'active'}
      <g class="node" role="button" tabindex="0"
         onclick={() => clickBaby(b, xy.x, xy.y)}
         onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') clickBaby(b, xy.x, xy.y); }}
         onmouseenter={() => { hovered = b.id; }} onmouseleave={() => { hovered = null; }}
         aria-label="baby step: {b.label}">
        {#if isBabyActive && allowMotion}
          <circle cx={xy.x} cy={xy.y} r={11} fill="none" stroke="var(--teal-bright)" stroke-width="1.5" opacity="0.65">
            <animate attributeName="r" values="9;18;9" dur="2.2s" repeatCount="indefinite" />
            <animate attributeName="opacity" values="0.7;0;0.7" dur="2.2s" repeatCount="indefinite" />
          </circle>
        {/if}
        <circle cx={xy.x} cy={xy.y} r={r} fill={colorFor(status)} filter={isBabyActive ? 'url(#bloom-strong)' : undefined}>
          {#if isBabyActive && allowMotion}
            <animate attributeName="opacity" values="0.55;1;0.55" dur="2.2s" repeatCount="indefinite" />
          {/if}
        </circle>
        {#if isBabyActive || hovered === b.id}
          <text x={xy.x + (xy.x > cx + 4 ? 12 : -12)} y={xy.y + 4}
                text-anchor={xy.x > cx + 4 ? 'start' : 'end'}
                class="label label--baby" class:label--active={isBabyActive}
                paint-order="stroke" stroke="var(--bg-base)" stroke-width="2.5" stroke-linejoin="round">
            {b.label.toLowerCase()}
          </text>
        {/if}
      </g>
    {/each}

    <!-- centre node — concept identity, halo coloured by FSM -->
    <circle cx={cx} cy={cy} r="22" fill="none" stroke={haloColor} stroke-width="0.5" opacity="0.18" class="halo-ambient" />
    <circle cx={cx} cy={cy} r="14" fill="none" stroke={haloColor} stroke-width="0.5" opacity="0.5" class="halo-pulse" />
    <circle cx={cx} cy={cy} r="9" fill="var(--bg-base)" stroke={haloColor} stroke-width="1.2" />
    {#if fixture.bubble_action}
      <circle cx={cx} cy={cy} r="18" fill="none" stroke="var(--bubble-{fixture.bubble_action})" stroke-width="1.5" opacity="0.7" class="bubble-ring" />
    {/if}
    <text x={cx} y={cy + 4} text-anchor="middle" class="centre-label">{concept.name.split(' ').map((w) => w[0]).join('').slice(0, 3).toLowerCase()}</text>

    <!-- GRC dots orbit the active baby -->
    {#if activeBabyPos && fixture.grc_step}
      <OrbitGrcDots cx={activeBabyPos.x} cy={activeBabyPos.y} active_step={fixture.grc_step} />
    {/if}

    <!-- Context-budget donut around centre, only at concept zoom -->
    {#if fixture.context_budget && zoomState.level === 'concept'}
      <OrbitBudgetRing cx={cx} cy={cy} radius={36} slots={fixture.context_budget} />
    {/if}
    {/if}
    </g>
  {/if}
</g>

<style>
  /* Hide the SVG decomposition tree — three.js owns the orbit now.
     The class wraps the active-concept content; toggling display
     re-enables the legacy SVG view for debugging. */
  .svg-orbit-content {
    display: none;
  }

  /* Inactive ambient edges: subtle dash-flow gives "synapse" feel. */
  .ambient-line {
    stroke-dasharray: 2 6;
    animation: ambient-flow 18s linear infinite;
  }
  @keyframes ambient-flow {
    from { stroke-dashoffset: 0; }
    to   { stroke-dashoffset: -16; }
  }

  .concept-group--inactive { opacity: 0.55; }
  .concept-group--inactive:hover { opacity: 0.85; cursor: pointer; }

  .node { cursor: pointer; }
  .node:hover circle { filter: brightness(1.25); }

  .label {
    font-family: var(--font-mono);
    fill: var(--text-secondary);
    letter-spacing: 0.04em;
  }
  .label--phase { font-size: 10px; }
  .label--step  { font-size: 9px; fill: var(--text-ghost); }
  .label--baby  { font-size: 9px; fill: var(--text-ghost); }
  .label--active  { fill: var(--teal-bright); }
  .label--ambient { fill: var(--text-ghost); opacity: 0.65; font-size: 9px; }

  .centre-label {
    font-family: var(--font-mono);
    font-size: 8px;
    fill: var(--teal-bright);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .active-line {
    stroke-dasharray: 4 3;
    animation: dashflow 1.6s linear infinite;
  }
  @keyframes dashflow { from { stroke-dashoffset: 0; } to { stroke-dashoffset: -14; } }

  .halo-pulse {
    transform-origin: center;
    transform-box: fill-box;
    animation: halo 4s ease-in-out infinite;
  }
  @keyframes halo {
    0%, 100% { opacity: 0.30; }
    50%      { opacity: 0.55; }
  }

  .halo-ambient {
    animation: ambient-breathe 7s ease-in-out infinite;
  }
  @keyframes ambient-breathe {
    0%, 100% { opacity: 0.10; r: 22; }
    50%      { opacity: 0.22; r: 26; }
  }

  .bubble-ring { animation: bubble-ripple 2.6s ease-out infinite; }
  @keyframes bubble-ripple {
    0%   { r: 14; opacity: 0.8; }
    70%  { r: 28; opacity: 0.0; }
    100% { r: 28; opacity: 0.0; }
  }

  @media (prefers-reduced-motion: reduce) {
    .active-line, .halo-pulse, .halo-ambient, .bubble-ring,
    .concept-group, .ambient-line { animation: none; }
    .active-line, .ambient-line { stroke-dasharray: none; }
  }
</style>
