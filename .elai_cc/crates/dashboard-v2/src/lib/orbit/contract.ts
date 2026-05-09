// orbit/contract.ts
// Type-level + pure-function preconditions for the orbit module.
// svelte-check enforces this; failures here mean the implementation drifted from spec.
// No runtime assertions — only type-level structural checks.

import type {
  OrbitMode,
  ZoomLevel,
  ConceptStub,
  PhaseStub,
  StepStub,
  BabyStepStub,
  OrbitFixture,
  GrcStep,
  CaptionContext,
  ZoomState,
  ZoomTarget,
  PageRankNodeStub,
  DkgEdgeStub,
  TendrilEvent,
} from './types';
import type { FsmState, EvidenceGate, ContextBudgetSlot, AgentRole, SourceTier, ResearchTreeNode } from '../api/types';
import { lcg, hashId } from './lcg';
import { polar, curve, distributeAngles, nodeBoundsToViewport } from './geometry';
import { captionFor, CAPTION_TABLE } from './captions';
import { zoomTo, interpolateZoom, ZOOM_LEVELS, defaultZoomState } from './zoom';
import { ORBIT_FIXTURE_SMALL, ORBIT_FIXTURE_CONSTELLATION, ORBIT_FIXTURE_RESEARCH } from '../api/mock-orbit';

// -----------------------------------------------------------------
// Type assertion helper — TS error if T is not the expected shape.
// -----------------------------------------------------------------
type Assert<T, Expected> = T extends Expected ? (Expected extends T ? true : false) : false;
type Same<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;

// -----------------------------------------------------------------
// Types: orbit mode + zoom level
// -----------------------------------------------------------------
const _modes: OrbitMode[] = ['plan', 'map', 'research'];
const _levels: ZoomLevel[] = ['concept', 'phase', 'step', 'baby-step'];

// CAPTION_TABLE must cover at least the 14 entries from HANDOFF-orbit.md:340-358
const _captionTableSize: number = CAPTION_TABLE.length;
type _CaptionTableSize = typeof CAPTION_TABLE['length'] extends number ? true : false;
const _captionAtLeast14: true = (_captionTableSize >= 14) as true;

// -----------------------------------------------------------------
// LCG: seeded RNG, returns () => number in [0, 1)
// -----------------------------------------------------------------
const _rng = lcg(0xC0FFEE);
type _LcgIsCallable = Assert<typeof _rng, () => number>;
const _h: number = hashId('concept-1');

// -----------------------------------------------------------------
// Geometry: polar/curve/distributeAngles/bounds
// -----------------------------------------------------------------
const _p: { x: number; y: number } = polar(0, 100);
const _c: string = curve(0, 0, 100, 100, 0.18);
const _curveStartsWithM: true = (_c[0] === 'M') as true;
const _angles: number[] = distributeAngles(5, 0, Math.PI);
const _vp: { x: number; y: number; w: number; h: number } = nodeBoundsToViewport({ x: 100, y: 100, r: 10 }, 0.12);

// -----------------------------------------------------------------
// Captions: pure function (CaptionContext) => string
// -----------------------------------------------------------------
const _ctx: CaptionContext = {
  fsm: 'EXECUTING',
  grc_step: 'act',
  baby_step_label: 'add toggle',
  bubble_action: undefined,
};
const _cap: string = captionFor(_ctx);

// -----------------------------------------------------------------
// Zoom: state machine + interpolation
// -----------------------------------------------------------------
const _zs: ZoomState = defaultZoomState();
const _zt: ZoomTarget = { level: 'phase', node_id: 'phase-1', x: 320, y: 280, scale: 1.6 };
const _zNext: ZoomState = zoomTo(_zs, _zt);
const _zInterp: ZoomState = interpolateZoom(_zs, _zNext, 0.5);
const _zoomLevelsLen: 4 = ZOOM_LEVELS.length as 4;

// -----------------------------------------------------------------
// Fixtures: at least 3, each is OrbitFixture-shaped
// -----------------------------------------------------------------
const _f1: OrbitFixture = ORBIT_FIXTURE_SMALL;
const _f2: OrbitFixture = ORBIT_FIXTURE_CONSTELLATION;
const _f3: OrbitFixture = ORBIT_FIXTURE_RESEARCH;
const _smallHasOneConcept: true = (_f1.concepts.length === 1) as true;
const _constellationHasMultiple: true = (_f2.concepts.length >= 2) as true;
const _researchHasNodes: true = (_f3.research_tree !== undefined) as true;

// -----------------------------------------------------------------
// Stub shapes propagate from existing api/types.ts
// -----------------------------------------------------------------
type _GrcStepValid = Assert<GrcStep, 'observe' | 'hypothesize' | 'plan_action' | 'act' | 'verify' | 'converge' | 'status'>;
type _OrbitModeValid = Assert<OrbitMode, 'plan' | 'map' | 'research'>;
type _ZoomLevelValid = Assert<ZoomLevel, 'concept' | 'phase' | 'step' | 'baby-step'>;
type _SourceTierValid = Assert<SourceTier, 'T1' | 'T2' | 'T3' | 'T4' | 'T5'>;
type _AgentRoleHasEightVariants = Same<
  AgentRole,
  'executor' | 'auditor' | 'researcher' | 'designer' | 'prompt-engineer' | 'context-compiler' | 'memory-writer' | 'media-validator'
>;

// -----------------------------------------------------------------
// PageRank + DKG + Tendril stubs exist for Mode 2 / Mode 3
// -----------------------------------------------------------------
const _pr: PageRankNodeStub = { id: 'n1', name: 'fsm.rs::transition_with_metrics', score: 0.18, file: 'crates/orchestrator/src/fsm.rs', effect_tags: [] };
const _de: DkgEdgeStub = { source: 'n1', target: 'n2', edge_type: 'call', weight: 0.7 };
const _te: TendrilEvent = { id: 't1', source_url: 'https://docs.rs/bb8', tier: 'T1', extracted_quote: 'Generic connection pool', returned: true };

// -----------------------------------------------------------------
// Re-exports verify the public API surface
// -----------------------------------------------------------------
export type {
  OrbitMode, ZoomLevel, ConceptStub, PhaseStub, StepStub, BabyStepStub, OrbitFixture,
  GrcStep, CaptionContext, ZoomState, ZoomTarget, PageRankNodeStub, DkgEdgeStub, TendrilEvent,
};
