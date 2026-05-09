// node-panel-variants.ts
//
// Registry of per-node UI content for the V3 hybrid NodePanel design:
//   • LAYER 1 — 3D Mandala (handled by NodeMandala class)
//   • LAYER 2 — 4 (or 5) compass MiniCards (DOM, screen-projected,
//                  camera-facing) — content from `compass`
//   • LAYER 3 — 1 main Workstation panel (DOM, anchored south of node) —
//                  content from `main`
//
// Each variant returns a `VariantContent` describing every layer's
// strings. NodeMandala receives flag/colour hints separately
// (e.g. `showNwMarker`, `tierColored`) — that's wired in OrbitCanvas.
//
// SYNTHETIC content is comment-marked so a follow-up plumbing pass can
// replace it with live SSB/FSM telemetry.

import type {
  OrbitFixture,
  ConceptStub,
  PhaseStub,
  StepStub,
  BabyStepStub,
  OrbitMode,
} from './types';
import type { ResearchTreeNode, EvidenceGate } from '../api/types';
import { type MandalaTextContent, emptyTextContent } from './node-mandala.ts';

// ---------- public types ----------

export type VariantKey =
  | 'concept'
  | 'phase'
  | 'step'
  | 'baby'
  | 'integration-point'
  | 'research-node'
  | 'fallback';

export interface MiniCardLine {
  text: string;
  classMod?: 'title' | 'mono' | 'mono-bright' | 'sans-dim';
}

export interface MiniCardContent {
  /** Header tag (without surrounding brackets). */
  tag: string;
  /** 1-2 content lines. */
  lines: MiniCardLine[];
}

/** Compass map — every variant ships at least IDENTITY + STATUS; the
 *  full set is N=identity, E=location, S=status, W=activity, plus the
 *  optional NW=dependencies for the integration-point variant. */
export interface CompassContent {
  n: MiniCardContent;
  e?: MiniCardContent;
  s: MiniCardContent;
  w?: MiniCardContent;
  nw?: MiniCardContent;
}

export type MainQuadrantId = 'identity' | 'liveActivity' | 'diff' | 'progress' | 'phasesList' | 'dependencies' | 'sourcesTier';

export interface MainQuadrant {
  /** Header tag (no brackets) — uppercase mono. */
  tag: string;
  /** Quadrant body lines. */
  lines: MiniCardLine[];
  /** Quadrant kind — controls extra rendering (radial progress arc,
   *  source-tier dot row). */
  kind: MainQuadrantId;
  /** Used by progress quadrant — fraction in [0,1]. */
  progressFrac?: number;
  /** Used by progress quadrant — meta rows. */
  metaRows?: Array<[string, string]>;
  /** Used by sourcesTier quadrant — colored dots per tier. */
  tierDots?: Array<'T1' | 'T2' | 'T3'>;
}

export interface MainPanelContent {
  /** Top-strip breadcrumb (mono uppercase, dim). */
  breadcrumb: string;
  /** Quadrants ordered TL, TR, BL, BR. */
  quadrants: [MainQuadrant, MainQuadrant, MainQuadrant, MainQuadrant];
}

export interface VariantContent {
  variantKey: VariantKey;
  /** Compass mini-cards. Some variants omit a slot (e.g. concept). */
  compass: CompassContent;
  /** Main workstation panel. */
  main: MainPanelContent;
  /** When true, NodeMandala renders the 5th NW marker. */
  showNwMarker: boolean;
  /** When true, NodeMandala recolours markers per source tier. */
  tierColored: boolean;
}

export interface NodePanelContext {
  fixture: OrbitFixture;
  mode: OrbitMode;
  nodeId: string;
  nodeType: 'phase' | 'step' | 'baby';
}

// ---------- helpers ----------

function findConcept(fx: OrbitFixture): ConceptStub | undefined {
  return fx.concepts.find((c) => c.id === fx.active_concept_id) ?? fx.concepts[0];
}
function findPhase(fx: OrbitFixture, id: string): { phase: PhaseStub; concept: ConceptStub; idx: number; total: number } | null {
  for (const c of fx.concepts) {
    const idx = c.phases.findIndex((x) => x.id === id);
    if (idx >= 0) return { phase: c.phases[idx], concept: c, idx, total: c.phases.length };
  }
  return null;
}
function findStep(fx: OrbitFixture, id: string): { step: StepStub; phase: PhaseStub; concept: ConceptStub; idx: number; total: number } | null {
  for (const c of fx.concepts) {
    for (const p of c.phases) {
      const idx = p.steps.findIndex((s) => s.id === id);
      if (idx >= 0) return { step: p.steps[idx], phase: p, concept: c, idx, total: p.steps.length };
    }
  }
  return null;
}
function findBaby(fx: OrbitFixture, id: string): { baby: BabyStepStub; step: StepStub; phase: PhaseStub; concept: ConceptStub; idx: number; total: number } | null {
  for (const c of fx.concepts) {
    for (const p of c.phases) {
      for (const s of p.steps) {
        const idx = s.baby_steps.findIndex((x) => x.id === id);
        if (idx >= 0) return { baby: s.baby_steps[idx], step: s, phase: p, concept: c, idx, total: s.baby_steps.length };
      }
    }
  }
  return null;
}
function findResearchNode(root: ResearchTreeNode | undefined, id: string): ResearchTreeNode | null {
  if (!root) return null;
  if (root.id === id) return root;
  for (const c of root.children ?? []) {
    const f = findResearchNode(c, id);
    if (f) return f;
  }
  return null;
}

function pad2(n: number): string { return String(Math.max(0, Math.floor(n))).padStart(2, '0'); }

function statusForFsm(fsm: OrbitFixture['fsm']): { glyph: string; text: string; tone: string } {
  switch (fsm) {
    case 'PLANNING':       return { glyph: '◐', text: 'PLANNING',  tone: 'planning' };
    case 'TESTING_FIRST':  return { glyph: '◐', text: 'TESTING',   tone: 'planning' };
    case 'RESEARCHING':    return { glyph: '◐', text: 'RESEARCH',  tone: 'planning' };
    case 'EXECUTING':      return { glyph: '⊙', text: 'EXECUTING', tone: 'executing' };
    case 'AUDITING':       return { glyph: '⊙', text: 'AUDITING',  tone: 'executing' };
    case 'MEMORIZING':     return { glyph: '⊙', text: 'MEMORIZE',  tone: 'executing' };
    case 'COMPLETE':       return { glyph: '●', text: 'DONE',      tone: 'done' };
    default:               return { glyph: '○', text: 'IDLE',      tone: 'idle' };
  }
}

function asciiBar(frac: number, cells = 9): string {
  const f = Math.max(0, Math.min(1, frac));
  const filled = Math.round(f * cells);
  return '▓'.repeat(filled) + '░'.repeat(cells - filled);
}

function gateGlyphs(gates: EvidenceGate[] | undefined): string {
  if (!gates || gates.length === 0) return '○ ○ ○ ○ ○';
  return gates.map((g) => {
    if (g.verdict === 'pass') return '●';
    if (g.verdict === 'warn') return '◐';
    if (g.verdict === 'block') return '⊙';
    return '○';
  }).join(' ');
}

// ---------- variants ----------

function conceptVariant(ctx: NodePanelContext): VariantContent {
  const c = findConcept(ctx.fixture);
  const title = c?.name ?? 'Concept';
  const desc = c?.description ?? ctx.fixture.idea ?? '';
  const phaseCount = c?.phases.length ?? 0;
  let stepCount = 0;
  for (const p of c?.phases ?? []) stepCount += p.steps.length;

  const activePhase = c?.phases.find((p) => p.id === ctx.fixture.active_phase_id);
  const activePhaseIdx = activePhase ? (c?.phases.findIndex((p) => p.id === activePhase.id) ?? -1) : -1;
  const activeStep = activePhase?.steps.find((s) => s.id === ctx.fixture.active_step_id);
  const activeStepIdx = activeStep ? (activePhase?.steps.findIndex((s) => s.id === activeStep.id) ?? -1) : -1;
  const activeLabel = activePhase && activeStep
    ? `phase ${pad2(activePhaseIdx + 1)} · step ${pad2(activeStepIdx + 1)}`
    : 'idle';

  const status = statusForFsm(ctx.fixture.fsm);
  // Concept fixture: ONLY the IDENTITY + STATUS mini-cards.
  return {
    variantKey: 'concept',
    showNwMarker: false,
    tierColored: false,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: title, classMod: 'title' },
          { text: 'project · concept root', classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `${status.glyph} ${status.text}`, classMod: 'mono-bright' },
          { text: activeLabel, classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `PROJECT-CONCEPT · ${phaseCount} PHASES · ${stepCount} STEPS`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: title, classMod: 'title' },
            { text: desc, classMod: 'sans-dim' },
            { text: 'concept root', classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          // SYNTHETIC: live LLM activity stream not wired yet.
          lines: [
            { text: `▶ ${status.text.toLowerCase()}`, classMod: 'mono-bright' },
            { text: activeLabel, classMod: 'mono' },
            { text: 'awaiting next phase boundary', classMod: 'mono' },
          ],
        },
        {
          // Concept variant replaces DIFF with PHASES list of phase names.
          tag: 'PHASES',
          kind: 'phasesList',
          lines: (c?.phases ?? []).slice(0, 5).map((p, i) => ({
            text: `${pad2(i + 1)} · ${p.label}`,
            classMod: i === activePhaseIdx ? 'mono-bright' : 'mono',
          })),
        },
        {
          tag: 'PROGRESS',
          kind: 'progress',
          lines: [],
          // SYNTHETIC: phase-completion ratio = active phase index / total.
          progressFrac: phaseCount > 0
            ? Math.max(0, Math.min(1, (activePhaseIdx + 1) / phaseCount))
            : 0,
          metaRows: [
            ['phases',  String(phaseCount)],
            ['steps',   String(stepCount)],
            ['active',  activeLabel],
          ],
        },
      ],
    },
  };
}

function phaseVariant(ctx: NodePanelContext): VariantContent {
  const found = findPhase(ctx.fixture, ctx.nodeId);
  const status = statusForFsm(ctx.fixture.fsm);
  if (!found) return fallbackVariant(ctx);
  const total = found.phase.steps.length;
  const activeStepIdx = found.phase.steps.findIndex((s) => s.id === ctx.fixture.active_step_id);
  const stepNum = activeStepIdx >= 0 ? activeStepIdx + 1 : 0;
  const frac = total > 0 ? stepNum / total : 0;
  const evidenceTotal = ctx.fixture.evidence_gates?.length ?? 0;
  const evidencePassed = ctx.fixture.evidence_gates?.filter((g) => g.verdict === 'pass').length ?? 0;
  const activeStep = found.phase.steps[activeStepIdx];
  // SYNTHETIC: live counters not present in OrbitFixture.
  const filesMod = `${(found.idx + 1) * 3 + total} mod`;
  const tokens = `${(2.0 + found.idx * 0.4 + total * 0.1).toFixed(1)}K tok`;
  const elapsed = `${pad2((found.idx * 17) % 60)}s`;
  return {
    variantKey: 'phase',
    showNwMarker: false,
    tierColored: false,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: found.phase.label, classMod: 'title' },
          { text: `phase ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
        ],
      },
      e: {
        tag: 'LOCATION',
        lines: [
          { text: found.concept.name, classMod: 'mono-bright' },
          { text: `concept · ${found.concept.phases.length} phases`, classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `${status.glyph} ${status.text}`, classMod: 'mono-bright' },
          { text: `${asciiBar(frac, 9)} ${Math.round(frac * 100)}%`, classMod: 'mono' },
        ],
      },
      w: {
        tag: 'ACTIVITY',
        lines: [
          // SYNTHETIC: replace with live FSM/SSB telemetry later.
          { text: activeStep ? `▶ ${activeStep.label.toLowerCase()}` : '▶ idle', classMod: 'mono-bright' },
          { text: `${tokens} · ${filesMod}`, classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `EX-${(new Date()).getFullYear()} · PHASE-${pad2(found.idx + 1)} / ${pad2(found.total)}`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: found.phase.label, classMod: 'title' },
            { text: `phase ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
            { text: `${found.concept.name} → ${total} steps`, classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          lines: [
            // SYNTHETIC: replace with live LLM action stream later.
            { text: `▶ active step: ${activeStep?.label ?? 'idle'}`, classMod: 'mono-bright' },
            { text: `${tokens} · ${filesMod}`, classMod: 'mono' },
            { text: `evidence ${evidencePassed}/${evidenceTotal}`, classMod: 'mono' },
            { text: 'reading 3 ctx', classMod: 'mono' },
          ],
        },
        {
          tag: 'DIFF',
          kind: 'diff',
          // SYNTHETIC: replace with live SSB diff plumbing later.
          lines: [
            { text: '+ fn pool_builder()', classMod: 'mono-bright' },
            { text: '+   self.cap = cap;', classMod: 'mono' },
            { text: '-   self.cap = 32;', classMod: 'mono' },
            { text: '~ struct PoolCfg', classMod: 'mono' },
          ],
        },
        {
          tag: 'PROGRESS',
          kind: 'progress',
          lines: [],
          progressFrac: frac,
          metaRows: [
            ['tokens',  tokens],
            ['elapsed', elapsed],
            ['tests',   `${evidencePassed}/${evidenceTotal} pass`],
          ],
        },
      ],
    },
  };
}

function stepVariant(ctx: NodePanelContext): VariantContent {
  const found = findStep(ctx.fixture, ctx.nodeId);
  if (!found) return fallbackVariant(ctx);
  const status = statusForFsm(ctx.fixture.fsm);
  const isActive = ctx.fixture.active_step_id === found.step.id;
  const firstBaby = found.step.baby_steps[0];
  const fileRef = firstBaby?.task ?? '';
  // SYNTHETIC: live LLM telemetry stub.
  const action = firstBaby ? `writing ${firstBaby.label.toLowerCase()}` : 'idle';
  const ctxLine = `${1 + (found.idx % 3)} ctx · ${(0.8 + found.idx * 0.2).toFixed(1)}K tok`;
  const evidenceTotal = ctx.fixture.evidence_gates?.length ?? 0;
  const evidencePassed = ctx.fixture.evidence_gates?.filter((g) => g.verdict === 'pass').length ?? 0;
  const frac = evidenceTotal > 0 ? evidencePassed / evidenceTotal : (isActive ? 0.45 : 0.0);

  return {
    variantKey: 'step',
    showNwMarker: false,
    tierColored: false,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: found.step.label, classMod: 'title' },
          { text: `step ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
        ],
      },
      e: {
        tag: 'LOCATION',
        lines: [
          { text: fileRef || 'no file', classMod: 'mono-bright' },
          { text: `${found.phase.label}`, classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `${status.glyph} ${status.text}`, classMod: 'mono-bright' },
          { text: gateGlyphs(ctx.fixture.evidence_gates), classMod: 'mono' },
        ],
      },
      w: {
        tag: 'ACTIVITY',
        lines: [
          { text: `▶ ${action}`, classMod: 'mono-bright' },
          { text: ctxLine, classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `EX-${(new Date()).getFullYear()} · PHASE-?? / STEP-${pad2(found.idx + 1)} / BABY-STEP-01`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: found.step.label, classMod: 'title' },
            { text: `step ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
            { text: `${found.phase.label} → ${found.concept.name}`, classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          lines: [
            { text: `▶ ${action}`, classMod: 'mono-bright' },
            { text: fileRef, classMod: 'mono' },
            { text: ctxLine, classMod: 'mono' },
            { text: 'logits stable', classMod: 'mono' },
          ],
        },
        {
          tag: 'DIFF',
          kind: 'diff',
          // SYNTHETIC: replace with live SSB diff plumbing later.
          lines: [
            { text: '+ pub fn handle_refresh()', classMod: 'mono-bright' },
            { text: '+   let claims = decode(t)?;', classMod: 'mono' },
            { text: '+   if claims.exp < now()...', classMod: 'mono' },
            { text: '~ trait RefreshGuard', classMod: 'mono' },
          ],
        },
        {
          tag: 'PROGRESS',
          kind: 'progress',
          lines: [],
          progressFrac: frac,
          metaRows: [
            ['tokens',  ctxLine.split(' · ')[1] ?? '0K'],
            ['elapsed', `${(found.idx + 1) * 4}s`],
            ['tests',   `${evidencePassed}/${evidenceTotal} pass`],
          ],
        },
      ],
    },
  };
}

function babyVariant(ctx: NodePanelContext): VariantContent {
  const found = findBaby(ctx.fixture, ctx.nodeId);
  if (!found) return fallbackVariant(ctx);
  const status = statusForFsm(ctx.fixture.fsm);
  const isActive = ctx.fixture.active_baby_step_id === found.baby.id;
  const fileRef = found.baby.task ?? '';
  // SYNTHETIC.
  const elapsed = isActive ? `${(2 + found.idx * 2)}s` : '—';
  const evidenceTotal = ctx.fixture.evidence_gates?.length ?? 0;
  const evidencePassed = ctx.fixture.evidence_gates?.filter((g) => g.verdict === 'pass').length ?? 0;
  const frac = evidenceTotal > 0 ? evidencePassed / evidenceTotal : (isActive ? 0.55 : 0);

  return {
    variantKey: 'baby',
    showNwMarker: false,
    tierColored: false,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: found.baby.label, classMod: 'title' },
          { text: `baby ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
        ],
      },
      e: {
        tag: 'LOCATION',
        lines: [
          { text: fileRef || 'no file', classMod: 'mono-bright' },
          { text: found.step.label, classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `${status.glyph} ${status.text}`, classMod: 'mono-bright' },
          { text: `elapsed ${elapsed}`, classMod: 'mono' },
        ],
      },
      w: {
        tag: 'ACTIVITY',
        lines: [
          // SYNTHETIC.
          { text: `▶ writing fn ${found.baby.label.toLowerCase().replace(/\s+/g, '_')}`, classMod: 'mono-bright' },
          { text: `${(0.6 + found.idx * 0.15).toFixed(1)}K tok · 2 ctx`, classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `EX-${(new Date()).getFullYear()} · STEP-?? / BABY-STEP-${pad2(found.idx + 1)}`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: found.baby.label, classMod: 'title' },
            { text: `baby ${pad2(found.idx + 1)} / ${pad2(found.total)}`, classMod: 'mono' },
            { text: `${found.step.label} → ${found.phase.label}`, classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          lines: [
            // SYNTHETIC.
            { text: `▶ writing fn ${found.baby.label.toLowerCase().replace(/\s+/g, '_')}`, classMod: 'mono-bright' },
            { text: fileRef, classMod: 'mono' },
            { text: `${(0.6 + found.idx * 0.15).toFixed(1)}K tok · 2 ctx`, classMod: 'mono' },
            { text: `elapsed ${elapsed}`, classMod: 'mono' },
          ],
        },
        {
          tag: 'DIFF',
          kind: 'diff',
          // SYNTHETIC.
          lines: [
            { text: '+ pub struct PoolBuilder', classMod: 'mono-bright' },
            { text: '+   cap: usize,', classMod: 'mono' },
            { text: '+   timeout: Duration,', classMod: 'mono' },
            { text: '+ }', classMod: 'mono' },
          ],
        },
        {
          tag: 'PROGRESS',
          kind: 'progress',
          lines: [],
          progressFrac: frac,
          metaRows: [
            ['tokens',  `${(0.6 + found.idx * 0.15).toFixed(1)}K`],
            ['elapsed', elapsed],
            ['tests',   `${evidencePassed}/${evidenceTotal} pass`],
          ],
        },
      ],
    },
  };
}

function integrationVariant(ctx: NodePanelContext): VariantContent {
  const node = ctx.fixture.pagerank_nodes?.find((n) => n.id === ctx.nodeId);
  if (!node) return fallbackVariant(ctx);
  const edges = ctx.fixture.pagerank_edges ?? [];
  const linked = new Set<string>();
  let traitImpls = 0;
  for (const e of edges) {
    if (e.source === node.id) linked.add(e.target);
    else if (e.target === node.id) linked.add(e.source);
    if (e.edge_type === 'type_ref') traitImpls += 1;
  }
  const fileById = new Map<string, string>();
  for (const n of ctx.fixture.pagerank_nodes ?? []) fileById.set(n.id, n.file);
  const linkedFiles = new Set<string>();
  for (const otherId of linked) {
    const f = fileById.get(otherId);
    if (f && f !== node.file) linkedFiles.add(f);
  }
  const fileList = [...linkedFiles];
  const top4 = fileList.slice(0, 4);
  const more = Math.max(0, fileList.length - 4);

  return {
    variantKey: 'integration-point',
    showNwMarker: true,
    tierColored: false,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: node.name, classMod: 'title' },
          { text: 'integration point', classMod: 'mono' },
        ],
      },
      e: {
        tag: 'LOCATION',
        lines: [
          { text: node.file, classMod: 'mono-bright' },
          { text: `${node.effect_tags.join(', ') || 'no tags'}`, classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `pagerank ${(node.score * 100).toFixed(1)}`, classMod: 'mono-bright' },
          { text: `${linked.size} linked · ${traitImpls} impls`, classMod: 'mono' },
        ],
      },
      w: {
        tag: 'ACTIVITY',
        lines: [
          { text: `↪ ${linked.size} callers`, classMod: 'mono-bright' },
          { text: `${traitImpls} trait impls`, classMod: 'mono' },
        ],
      },
      nw: {
        tag: 'DEPENDENCIES',
        lines: [
          { text: `${linkedFiles.size} files linked`, classMod: 'mono-bright' },
          { text: top4[0] ?? '(none)', classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `MAP · INTEGRATION-POINT · ${node.effect_tags.join(' · ').toUpperCase() || 'NO-TAGS'}`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: node.name, classMod: 'title' },
            { text: 'integration point', classMod: 'mono' },
            { text: `pagerank ${(node.score * 100).toFixed(2)}`, classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          lines: [
            { text: `▶ idle (no live exec)`, classMod: 'mono-bright' },
            { text: `${linked.size} callers · ${traitImpls} impls`, classMod: 'mono' },
            { text: `effects: ${node.effect_tags.join(', ') || '—'}`, classMod: 'mono' },
            { text: node.file, classMod: 'mono' },
          ],
        },
        {
          // Map-mode BL: linked files list (no diff).
          tag: 'DEPENDENCIES',
          kind: 'dependencies',
          lines: [
            ...top4.map((f) => ({ text: f, classMod: 'mono' as const })),
            ...(more > 0 ? [{ text: `+ ${more} more`, classMod: 'sans-dim' as const }] : []),
          ],
        },
        {
          tag: 'PROGRESS',
          kind: 'progress',
          lines: [],
          progressFrac: Math.max(0, Math.min(1, node.score * 5)), // pagerank ∈ ~[0,0.2] → bar [0,1]
          metaRows: [
            ['callers', String(linked.size)],
            ['impls',   String(traitImpls)],
            ['files',   String(linkedFiles.size)],
          ],
        },
      ],
    },
  };
}

function researchVariant(ctx: NodePanelContext): VariantContent {
  const r = findResearchNode(ctx.fixture.research_tree, ctx.nodeId);
  if (!r) return fallbackVariant(ctx);
  const tier: 'T1' | 'T2' | 'T3' = r.stage === 'CONVERGE' ? 'T1'
    : r.stage === 'REFLECT' || r.stage === 'EXTRACT' ? 'T2' : 'T3';
  const stageGlyph = r.stage === 'CONVERGE' ? '●'
    : r.stage === 'REFLECT' ? '⊙'
    : r.stage === 'EXTRACT' ? '◐' : '○';
  // SYNTHETIC.
  const convergedCount = r.converged ? Math.max(1, Math.floor(r.source_count / 2)) : 0;
  const convergePct = r.source_count > 0 ? convergedCount / r.source_count : 0;
  const reflection = r.converged ? 'token rotation OK' : 'gathering';
  // SYNTHETIC: tier-dot row stub.
  const tierDots: Array<'T1' | 'T2' | 'T3'> = [];
  for (let i = 0; i < r.source_count && i < 6; i++) {
    tierDots.push(i < convergedCount ? 'T1' : (i % 2 === 0 ? 'T2' : 'T3'));
  }

  return {
    variantKey: 'research-node',
    showNwMarker: false,
    tierColored: true,
    compass: {
      n: {
        tag: 'IDENTITY',
        lines: [
          { text: `“${r.query}”`, classMod: 'title' },
          { text: `research · ${tier}`, classMod: 'mono' },
        ],
      },
      e: {
        tag: 'LOCATION',
        lines: [
          { text: `${r.source_count} sources`, classMod: 'mono-bright' },
          { text: `${convergedCount} converged`, classMod: 'mono' },
        ],
      },
      s: {
        tag: 'STATUS',
        lines: [
          { text: `${stageGlyph} ${r.stage.toUpperCase()}`, classMod: 'mono-bright' },
          { text: `tier ${tier}`, classMod: 'mono' },
        ],
      },
      w: {
        tag: 'ACTIVITY',
        lines: [
          { text: `▶ ${reflection}`, classMod: 'mono-bright' },
          { text: 'reflection in flight', classMod: 'mono' },
        ],
      },
    },
    main: {
      breadcrumb: `RESEARCH · ${tier} · ${r.stage.toUpperCase()}`,
      quadrants: [
        {
          tag: 'IDENTITY',
          kind: 'identity',
          lines: [
            { text: `“${r.query}”`, classMod: 'title' },
            { text: `research node · ${tier}`, classMod: 'mono' },
            { text: `stage: ${r.stage}`, classMod: 'mono' },
          ],
        },
        {
          tag: 'LIVE ACTIVITY',
          kind: 'liveActivity',
          lines: [
            { text: `▶ reflection: ${reflection}`, classMod: 'mono-bright' },
            { text: `${r.source_count} sources fetched`, classMod: 'mono' },
            { text: `${convergedCount} converged · ${r.source_count - convergedCount} pending`, classMod: 'mono' },
            { text: r.converged ? 'ready to extract' : 'waiting on more', classMod: 'mono' },
          ],
        },
        {
          // Research BL: source-tier dots.
          tag: 'SOURCES',
          kind: 'sourcesTier',
          lines: tierDots.map((t, i) => ({
            text: `${t} · src ${i + 1}`,
            classMod: t === 'T1' ? 'mono-bright' : 'mono',
          })),
          tierDots,
        },
        {
          tag: 'CONVERGENCE',
          kind: 'progress',
          lines: [],
          progressFrac: convergePct,
          metaRows: [
            ['sources',   String(r.source_count)],
            ['converged', String(convergedCount)],
            ['stage',     r.stage],
          ],
        },
      ],
    },
  };
}

function fallbackVariant(ctx: NodePanelContext): VariantContent {
  return {
    variantKey: 'fallback',
    showNwMarker: false,
    tierColored: false,
    compass: {
      n: { tag: 'IDENTITY', lines: [{ text: ctx.nodeId, classMod: 'title' }] },
      s: { tag: 'STATUS', lines: [{ text: '○ unknown', classMod: 'mono' }] },
    },
    main: {
      breadcrumb: 'UNKNOWN',
      quadrants: [
        { tag: 'IDENTITY', kind: 'identity', lines: [{ text: ctx.nodeId, classMod: 'title' }] },
        { tag: 'LIVE ACTIVITY', kind: 'liveActivity', lines: [{ text: '▶ idle', classMod: 'mono' }] },
        { tag: 'DIFF', kind: 'diff', lines: [{ text: '— no data —', classMod: 'mono' }] },
        { tag: 'PROGRESS', kind: 'progress', lines: [], progressFrac: 0, metaRows: [['—', '—']] },
      ],
    },
  };
}

// ---------- registry ----------

export function resolveVariant(ctx: NodePanelContext): VariantContent {
  if (ctx.nodeId === '__sun__' || ctx.nodeId === ctx.fixture.active_concept_id) {
    return conceptVariant(ctx);
  }
  if (ctx.mode === 'research') return researchVariant(ctx);
  if (ctx.mode === 'map')      return integrationVariant(ctx);
  if (ctx.nodeType === 'phase') return phaseVariant(ctx);
  if (ctx.nodeType === 'step')  return stepVariant(ctx);
  if (ctx.nodeType === 'baby')  return babyVariant(ctx);
  return fallbackVariant(ctx);
}

// ---------- 3D-text projection ----------
//
// flattenVariantText takes the existing CompassContent + MainPanelContent
// and projects them into the FLAT MandalaTextContent shape the 3D troika
// labels need. Single source of truth — variants still produce structured
// data; the 3D text layer reads the right slots from that data without
// the variants needing to know about 3D.

export function flattenVariantText(v: VariantContent | null): MandalaTextContent {
  const out = emptyTextContent();
  if (!v) return out;
  const c = v.compass;
  // IDENTITY card → title + subtitle
  out.title    = c.n.lines[0]?.text ?? '';
  out.subtitle = c.n.lines[1]?.text ?? '';
  // STATUS card → status + elapsed
  out.status   = c.s.lines[0]?.text ?? '';
  out.elapsed  = c.s.lines[1]?.text ?? '';
  // LOCATION card (e) → fileRef
  out.fileRef  = c.e?.lines[0]?.text ?? '';
  // ACTIVITY card (w) → liveAction + meta
  out.liveAction = c.w?.lines[0]?.text ?? '';
  out.meta       = c.w?.lines[1]?.text ?? '';
  // breadcrumb is verbatim from main
  out.breadcrumb = v.main.breadcrumb;
  // progressPercent is overridden by StarCanvas using the live progress
  // value to keep the inner ring + central text in lockstep, but we
  // pre-fill from the variant in case the StarCanvas pipe is stale.
  const progressQ = v.main.quadrants.find((q) => q.kind === 'progress');
  if (progressQ?.progressFrac !== undefined) {
    out.progressPercent = `${Math.round(progressQ.progressFrac * 100)}%`;
  }
  return out;
}
