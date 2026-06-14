// system_mode_fixtures.ts — MOCK fixtures for the J-31 system-composition
// mode UI shell (docs/thefinalglance/J-plan/J-31_canvas_system_mode.md).
//
// The J-31 BACKEND (orchestrator/src/system_canvas.rs + SystemContractEvent
// SSB) is NOT built. This module is the mock data the SystemMode renderers
// build against so the surface is ready to wire when the backend lands.
// When the real Tauri commands exist (bind_node / declare_edge / set_display
// + a get_system_canvas read), swap these fixtures for a tryInvoke() call —
// the renderer shapes stay identical.
//
// SHAPES mirror the J-31 spec exactly:
//   SystemNode { node_id, binding, display, lifecycle }
//   SystemEdge { producer_node, consumer_node, contract, verdict }
// The LLM NEVER sets a verdict (J-31 TD-3); verdicts derive from checker +
// journal rows. These fixtures stand in for that derivation.

/** A node binds to a real buildable unit (J-31 leg 2). DRAFT = unbound. */
export type SystemBindingKind = 'plan_unit' | 'contract' | 'mcp_service' | 'draft';

/** Node lifecycle (J-31 leg 4). */
export type SystemNodeLifecycle =
  | 'draft'      // unbound — renders grey
  | 'framed'     // bound; dictated intent passed the IdeaFramer gate
  | 'building'   // FSM active on the bound unit (live halo)
  | 'wired'      // producer symbol exists + consumer references it
  | 'verified'   // the contract's observable_effect SSB row exists
  | 'drifted';   // a blast-radius cone touched the bound symbols

/** Edge verdict ladder (J-31 leg 3), all DERIVED — never LLM-set. */
export type SystemEdgeVerdict = 'declared' | 'wired' | 'verified' | 'broken';

export interface SystemBinding {
  kind: SystemBindingKind;
  /** The bound id (plan-unit id / contract id / MCP server id). '' when draft. */
  ref: string;
}

export interface SystemNode {
  node_id: string;
  display: string;
  binding: SystemBinding;
  lifecycle: SystemNodeLifecycle;
  /** Grid position (the layout algorithm owns geometry; these are seeds). */
  col: number;
  row: number;
  /** For MCP nodes: the blake3 schema pin status (E-24 / G-4 rug-pull). */
  schemaPin?: 'pinned' | 'repinned_ok' | 'rug_pull';
  /** True when ingress from this node carries G-1 provenance taint. */
  tainted?: boolean;
}

export interface SystemEdgeContract {
  /** The exact field set wiring_contract_check.py parses. */
  produces: string;
  consumes: string;
  reachable_from: string;
  observable_effect: string;
  /** The observable_effect SSB row id, present once the edge is verified. */
  evidenceRow?: string;
}

export interface SystemEdge {
  producer_node: string;
  consumer_node: string;
  contract: SystemEdgeContract;
  verdict: SystemEdgeVerdict;
}

export interface SystemCanvas {
  nodes: SystemNode[];
  edges: SystemEdge[];
}

// A worked fixture: a trading system (maintainer's own example — backtester /
// data layer / live connector), exercising every lifecycle + verdict so the
// renderer's state coverage is provable from one fixture.
export const SYSTEM_CANVAS_FIXTURE: SystemCanvas = {
  nodes: [
    {
      node_id: 'n_data',
      display: 'Market Data Layer',
      binding: { kind: 'plan_unit', ref: 'concept-2.phase-1' },
      lifecycle: 'verified',
      col: 0,
      row: 1,
    },
    {
      node_id: 'n_feed',
      display: 'Exchange Feed (MCP)',
      binding: { kind: 'mcp_service', ref: 'mcp:binance-md@1' },
      lifecycle: 'wired',
      col: 0,
      row: 0,
      schemaPin: 'pinned',
      tainted: true,
    },
    {
      node_id: 'n_backtest',
      display: 'Backtester',
      binding: { kind: 'plan_unit', ref: 'concept-3.phase-2.step-4' },
      lifecycle: 'building',
      col: 1,
      row: 1,
    },
    {
      node_id: 'n_strategy',
      display: 'Strategy Contract',
      binding: { kind: 'contract', ref: 'contract:signal_emit_v2' },
      lifecycle: 'framed',
      col: 1,
      row: 0,
    },
    {
      node_id: 'n_live',
      display: 'Live Connector',
      binding: { kind: 'draft', ref: '' },
      lifecycle: 'draft',
      col: 2,
      row: 1,
    },
    {
      node_id: 'n_risk',
      display: 'Risk Gate',
      binding: { kind: 'plan_unit', ref: 'concept-4.phase-1' },
      lifecycle: 'drifted',
      col: 2,
      row: 0,
    },
  ],
  edges: [
    {
      producer_node: 'n_feed',
      consumer_node: 'n_data',
      contract: {
        produces: 'OhlcvFrame',
        consumes: 'OhlcvFrame',
        reachable_from: 'data_layer::ingest',
        observable_effect: 'md_frame_ingested',
        evidenceRow: 'ssb#41207',
      },
      verdict: 'verified',
    },
    {
      producer_node: 'n_data',
      consumer_node: 'n_backtest',
      contract: {
        produces: 'OhlcvFrame',
        consumes: 'OhlcvFrame',
        reachable_from: 'backtester::run',
        observable_effect: 'backtest_started',
      },
      verdict: 'wired',
    },
    {
      producer_node: 'n_strategy',
      consumer_node: 'n_backtest',
      contract: {
        produces: 'Signal',
        consumes: 'Signal',
        reachable_from: 'backtester::apply_signal',
        observable_effect: 'signal_applied',
      },
      verdict: 'declared',
    },
    {
      producer_node: 'n_backtest',
      consumer_node: 'n_risk',
      contract: {
        produces: 'Order',
        consumes: 'Order',
        reachable_from: 'risk_gate::check',
        observable_effect: 'order_screened',
      },
      verdict: 'broken',
    },
  ],
};

/** Human labels for the binding kinds (used in the inspector). */
export const BINDING_LABEL: Record<SystemBindingKind, string> = {
  plan_unit: 'PLAN UNIT',
  contract: 'CONTRACT',
  mcp_service: 'MCP SERVICE',
  draft: 'UNBOUND',
};

/** One-line plain-language description of each edge verdict (the EXPLAIN
 *  layer for wiring truth — cached strings, no LLM, B-13 pattern). */
export const VERDICT_CAPTION: Record<SystemEdgeVerdict, string> = {
  declared: 'authored — not yet checked against the code',
  wired: 'the producer exists and the consumer references it (static leg passed)',
  verified: 'a real run left the contract’s observable-effect row in the journal',
  broken: 'a previously-passing wiring leg now fails — re-audit needed',
};

/** Map an edge verdict to a quinary glyph state for NodeStateGlyph. */
export function verdictToQuinary(v: SystemEdgeVerdict): 'done' | 'active' | 'pending' | 'warn' | 'fail' {
  switch (v) {
    case 'verified': return 'done';
    case 'wired': return 'active';
    case 'declared': return 'pending';
    case 'broken': return 'fail';
  }
}

/** Map a node lifecycle to a quinary glyph state. */
export function lifecycleToQuinary(l: SystemNodeLifecycle): 'done' | 'active' | 'pending' | 'warn' | 'fail' {
  switch (l) {
    case 'verified': return 'done';
    case 'wired': return 'done';
    case 'building': return 'active';
    case 'framed': return 'pending';
    case 'draft': return 'pending';
    case 'drifted': return 'warn';
  }
}
