// orbit/types.ts
// Orbit-specific types layered on top of api/types.ts.
// Existing types we reuse (do not duplicate): FsmState, DecompNode, EvidenceGate,
// ResearchTreeNode, SourceTier, AgentRole, ContextBudgetSlot, GrcBudget,
// BabyStepBlock, BubbleUpEvent, AntiSpiralCounter, DagRepairEvent.

import type {
  FsmState,
  EvidenceGate,
  ResearchTreeNode,
  SourceTier,
  AgentRole,
  GrcBudget,
  ContextBudgetSlot,
} from '../api/types';

// -----------------------------------------------------------------
// Mode + zoom
// -----------------------------------------------------------------
export type OrbitMode = 'plan' | 'map' | 'research';

export type ZoomLevel = 'concept' | 'phase' | 'step' | 'baby-step';

export interface ZoomTarget {
  level: ZoomLevel;
  node_id: string;
  x: number;
  y: number;
  scale: number;
}

export interface ZoomState {
  level: ZoomLevel;
  cx: number;     // viewport center x in svg coords
  cy: number;     // viewport center y in svg coords
  scale: number;  // 1 = fit-to-canvas; > 1 = zoomed in
}

// -----------------------------------------------------------------
// GRC step — extends api/types ReasoningPayload step_type with 'converge'
// to match Rust GrcState (engine.rs).
// -----------------------------------------------------------------
export type GrcStep =
  | 'observe'
  | 'hypothesize'
  | 'plan_action'
  | 'act'
  | 'verify'
  | 'converge'
  | 'status';

// -----------------------------------------------------------------
// BubbleUp action variants (mirrors Rust BubbleUpAction).
// -----------------------------------------------------------------
export type BubbleUpAction =
  | 'research_alternative'
  | 'redecompose_step'
  | 'replan_phase'
  | 'replan_concept'
  | 'diagnose_and_retry'
  | 'escalate';

// -----------------------------------------------------------------
// Caption inputs — pure context the captionFor() function consumes.
// -----------------------------------------------------------------
export interface CaptionContext {
  fsm: FsmState;
  grc_step: GrcStep;
  baby_step_label: string;
  bubble_action?: BubbleUpAction;
  research_topic?: string;  // for RESEARCHING captions
}

// -----------------------------------------------------------------
// Decomposition stubs — light-weight twins of the rust ConceptNode/etc.
// We use these directly in mock-orbit fixtures so the orbit can render
// without going through the heavier api/types DecompNode (which conflates
// status state). These carry only structure; status/active state is
// computed by the orbit from fsm + active_baby_step_id.
// -----------------------------------------------------------------
export interface BabyStepStub {
  id: string;
  label: string;          // "Write the toggle component"
  task: string;           // longer description
  estimated_tokens?: number;
  intent?: BubbleUpAction; // optional override (e.g. for bubble-up overrides)
}

export interface StepStub {
  id: string;
  label: string;
  baby_steps: BabyStepStub[];
}

export interface PhaseStub {
  id: string;
  label: string;
  steps: StepStub[];
  depends_on?: string[];   // concept_ids
}

export interface ConceptStub {
  id: string;
  name: string;
  description?: string;
  phases: PhaseStub[];
}

// -----------------------------------------------------------------
// OrbitFixture — top-level mock data for one task.
// -----------------------------------------------------------------
export interface OrbitFixture {
  task_id: string;          // hashed → LCG seed for deterministic positions
  idea: string;             // human-readable task description
  fsm: FsmState;            // current state
  active_concept_id?: string;
  active_phase_id?: string;
  active_step_id?: string;
  active_baby_step_id?: string;
  grc_step?: GrcStep;
  bubble_action?: BubbleUpAction;
  concepts: ConceptStub[];
  evidence_gates?: EvidenceGate[];
  context_budget?: ContextBudgetSlot[];
  agent_budgets?: GrcBudget[];
  research_tree?: ResearchTreeNode;       // mode 3 only
  pagerank_nodes?: PageRankNodeStub[];   // mode 2 only
  pagerank_edges?: DkgEdgeStub[];         // mode 2 only
  active_agent?: AgentRole;
}

// -----------------------------------------------------------------
// Mode 2 — DKG + PageRank stubs.
// Rust source: crates/knowledge/src/dkg.rs + crates/knowledge/src/pagerank/mod.rs.
// score range: practically [0.001, 0.2] post-normalization.
// effect_tags: subset of ['unsafe','filesystem','network','database','auth'].
// -----------------------------------------------------------------
export type DkgEffectTag = 'unsafe' | 'filesystem' | 'network' | 'database' | 'auth';

export type DkgEdgeType = 'call' | 'def' | 'import' | 'alias' | 'data_dep' | 'type_ref';

export interface PageRankNodeStub {
  id: string;
  name: string;             // qualified symbol name
  score: number;            // [0, 1] but practically [0.001, 0.2]
  file: string;
  effect_tags: DkgEffectTag[];
}

export interface DkgEdgeStub {
  source: string;
  target: string;
  edge_type: DkgEdgeType;
  weight: number;           // [0, 1]
}

// -----------------------------------------------------------------
// Mode 3 — research tendril event.
// Fires when a research-tree node sends out a SEARCH/EXTRACT and a
// source returns. The orbit visualises this as a tendril extending
// outside the orbit ring with a returning particle colored by tier.
// -----------------------------------------------------------------
export interface TendrilEvent {
  id: string;
  source_url: string;
  tier: SourceTier;
  extracted_quote?: string;
  returned: boolean;        // animation state: false=outbound, true=returning
  contradiction_with?: string;  // tendril id of contradicting source
}

// -----------------------------------------------------------------
// Idle VFX — shooting star
// -----------------------------------------------------------------
export interface ShootingStarConfig {
  enabled: boolean;
  fire_period_seconds: number;   // ~14s default
  trail_length: number;          // svg units
  intensity: number;             // 0..1
}

// -----------------------------------------------------------------
// Constellation layout (multi-concept).
// At zoom level 'concept', concepts arrange on an outer ring at
// golden-angle increments. At deeper zooms, the active concept centers
// and others fade.
// -----------------------------------------------------------------
export interface ConstellationLayout {
  concept_id: string;
  cx: number;
  cy: number;
  radius_scale: number;     // 1 if alone; <1 if collision-shrunk
}
