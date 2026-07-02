// ============================================================
// drawers.svelte.ts — Per-drawer live polling state.
//
// Phase C wiring: this file used to be a thin mock-data façade. It is now a
// set of $state-backed stores, each polling its own subset of stub.ts /
// Tauri commands at a 10-second cadence. Drawers call `xxxData.bootstrap()`
// in onMount and `xxxData.teardown()` in the cleanup return so we never
// poll a drawer the user can't see.
//
// Bootstrap is ref-counted so opening the same drawer from multiple places
// (or re-mounting due to layout changes) does not multiply timers.
//
// Each store also exposes an `unwired` object listing the data sections
// whose backend producers are NOT yet emitting real signals. Drawers consume
// this flag to render <UnwiredNote /> instead of zero-shaped placeholders.
// Phase numbers in `unwired` reference docs/thefinalglance/B-plan/B-0_INDEX.md.
// ============================================================

import * as stub from '../api/stub';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AgentEntry,
  AntiSpiralCounter,
  BspSuperstep,
  ContextBudget,
  CsqEntry,
  DepAuditResult,
  DeprecatedPattern,
  DkgStats,
  EmbeddingBreaker,
  EvidenceGate,
  FailureTaxonomyResult,
  FsmSnapshot,
  GgSignals,
  GroundingSweep,
  GrcBudget,
  HookExecution,
  HeartbeatEntry,
  InferenceMetrics,
  McpStatus,
  MistakeEntry,
  ProjectProfile,
  QueryTrace,
  RecallMetrics,
  ResearchTreeNode,
  ReflectState,
  Rule,
  RuleLifecycleCounts,
  SessionBranch,
  SidecarStatus,
  SourceTierDistribution,
  SupervisionEvent,
  VramGauge,
  WebAragState,
  ShortCoderStats,
  BuildSlot,
  DevServer,
  ScaffoldTemplate,
  SearchMode,
  BaselineResults,
  LintFinding,
  LighthouseReport,
  // E-17 SP4 — RAG retrieval-quality beacon row types.
  DeepPoolCappedRow,
  RrfQualityGateRow,
  HydeSkippedCodeQueryRow,
  RrfComponentWeightsRow,
  // B-4 — HyDE hypothesis-preview row for the QueryTrace inspector.
  HydeQueryRow,
  // B-5 — full CSQ verdict log row + per-role GRC config row.
  CsqLogRow,
  GrcConfigRow,
  // B-5 GAP-1 — anti-rubber-stamp rejection log row.
  AuditorRubberStampRejectionRow,
  // B-3 #11 — RulesDrawer rule_transition feed.
  RuleTransitionRow,
  // F-8 / Slice A5 — SCF symbolic-conflict feed row.
  SymbolicConflict,
  // F-11 — config-DGM admit-loop feed row.
  DarwinianAdmit,
  // H-46 — CMP parent-selection feed row.
  CladeSelection,
} from '../api/types';

const POLL_INTERVAL_MS = 10_000;

interface RefCountedTimer {
  count: number;
  timer: ReturnType<typeof setInterval> | null;
}

function makeRefCounted(refresh: () => Promise<void>): {
  bootstrap: () => void;
  teardown: () => void;
} {
  const ctx: RefCountedTimer = { count: 0, timer: null };
  return {
    bootstrap() {
      ctx.count += 1;
      if (ctx.timer === null) {
        void refresh();
        ctx.timer = setInterval(() => { void refresh(); }, POLL_INTERVAL_MS);
      }
    },
    teardown() {
      ctx.count = Math.max(0, ctx.count - 1);
      if (ctx.count === 0 && ctx.timer !== null) {
        clearInterval(ctx.timer);
        ctx.timer = null;
      }
    },
  };
}

// ------------------------------------------------------------------
// KNOWLEDGE drawer
// ------------------------------------------------------------------
// B-30: LanguageRegistry status shape from get_language_grammar_status Tauri command.
interface LanguageStatus {
  loaded: string[];
  failed: [string, string][];
}

// E-17 SP4: default beacon history window. The four feeds are narrow
// per-event SSB rows; 10 is enough for a sparkline + last-event summary
// without putting pressure on the IPC boundary at 10s cadence.
const RAG_QUALITY_BEACON_LIMIT = 10;

// E-17 SP4: shared loading / error state used by the four RAG retrieval-
// quality cards. A single error flag keeps the surface simple — the
// underlying invokes share an IPC channel, so they fail together when
// the Tauri runtime is unreachable.
type RagQualityLoadState = 'idle' | 'loading' | 'ready' | 'error';

function createKnowledge() {
  let dkgStats = $state<DkgStats>({ nodes: 0, edges: 0, bm25_chunks: 0, vector_status: 'BUILDING', freshness_decay_pct: 0, failed_files: 0 });
  let recentTraces = $state<QueryTrace[]>([]);
  // B-4: HyDE hypothesis-preview feed (newest-first), shown inline in the
  // QueryTrace inspector. Empty in steady state until a HyDE row lands.
  let recentHydeQueries = $state<HydeQueryRow[]>([]);
  let breakers = $state<EmbeddingBreaker[]>([]);
  let recallMetrics = $state<RecallMetrics[]>([]);
  let searchMode = $state<SearchMode>('HYBRID');
  // B-30: language grammar registry status (undefined until Tauri command is wired).
  let languageStatus = $state<LanguageStatus | undefined>(undefined);
  // E-17 SP4: RAG retrieval-quality beacons. Each array holds the most
  // recent N events newest-first (mirrors SsbViewer::recent_rows_of_type).
  let deepPoolCapped = $state<DeepPoolCappedRow[]>([]);
  let rrfQualityGate = $state<RrfQualityGateRow[]>([]);
  let hydeSkippedCodeQuery = $state<HydeSkippedCodeQueryRow[]>([]);
  let rrfComponentWeights = $state<RrfComponentWeightsRow[]>([]);
  let ragQualityLoadState = $state<RagQualityLoadState>('idle');
  let ragQualityLastError = $state<string | null>(null);

  async function refresh() {
    try {
      // E-17 SP4: mark loading on the first cycle so the cards render a
      // skeleton instead of an empty state. Subsequent cycles keep
      // showing the last data while a new one loads in the background.
      if (ragQualityLoadState === 'idle') {
        ragQualityLoadState = 'loading';
      }
      const [
        stats,
        traces,
        hyde,
        brk,
        recall,
        sm,
        deepCapped,
        rrfGate,
        hydeSkipped,
        rrfWeights,
      ] = await Promise.all([
        stub.get_dkg_stats(),
        stub.get_query_trace_recent(5),
        // B-4: HyDE hypothesis-preview rows for the QueryTrace inspector.
        stub.get_hyde_query_recent(5),
        stub.get_embedding_breakers(),
        stub.get_recall_metrics(),
        stub.get_search_mode(),
        // E-17 SP4: the 4 RAG retrieval-quality beacons share the
        // 10s knowledgeData cadence; each is a thin SsbViewer read with
        // a dashboard-side hard cap of 200 rows. Steady-state empty for
        // most sessions (rows arrive only when DEEP queries trip caps).
        stub.get_deep_pool_capped_recent(RAG_QUALITY_BEACON_LIMIT),
        stub.get_rrf_quality_gate_recent(RAG_QUALITY_BEACON_LIMIT),
        stub.get_hyde_skipped_code_query_recent(RAG_QUALITY_BEACON_LIMIT),
        stub.get_rrf_component_weights_recent(RAG_QUALITY_BEACON_LIMIT),
      ]);
      dkgStats = stats;
      recentTraces = traces;
      recentHydeQueries = hyde;
      breakers = brk;
      recallMetrics = recall;
      searchMode = sm.mode;
      deepPoolCapped = deepCapped;
      rrfQualityGate = rrfGate;
      hydeSkippedCodeQuery = hydeSkipped;
      rrfComponentWeights = rrfWeights;
      ragQualityLoadState = 'ready';
      ragQualityLastError = null;
      // Pass D: get_language_grammar_status wired — reads most recent SSB
      // language_grammar_status row via SsbViewer::latest_action_of_type.
      await stub.get_language_grammar_status()
        .then(r => { languageStatus = r; })
        .catch(() => { languageStatus = { loaded: [], failed: [] }; });
    } catch (err) {
      console.debug('[knowledgeData] refresh failed', err);
      // E-17 SP4: surface the error so the cards can render a retry
      // banner instead of a stale "no events yet" empty state.
      ragQualityLoadState = 'error';
      ragQualityLastError = err instanceof Error ? err.message : String(err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get dkgStats() { return dkgStats; },
    get recentTraces() { return recentTraces; },
    /** B-4: HyDE hypothesis-preview rows (newest-first) for the QueryTrace inspector. */
    get recentHydeQueries() { return recentHydeQueries; },
    get breakers() { return breakers; },
    get recallMetrics() { return recallMetrics; },
    get searchMode() { return searchMode; },
    /** B-30 Pass D: language grammar registry status. Wired in Pass D. */
    get languageStatus() { return languageStatus; },
    // E-17 SP4: RAG retrieval-quality beacons (read-only).
    get deepPoolCapped() { return deepPoolCapped; },
    get rrfQualityGate() { return rrfQualityGate; },
    get hydeSkippedCodeQuery() { return hydeSkippedCodeQuery; },
    get rrfComponentWeights() { return rrfComponentWeights; },
    get ragQualityLoadState() { return ragQualityLoadState; },
    get ragQualityLastError() { return ragQualityLastError; },
    /** Bucket-B sections — backend producer not yet wired. */
    unwired: {
      recentTraces: false,   // C-1: get_query_trace_recent reads SSB query_trace rows
      searchMode: false,     // C-8: get_search_mode reads SSB search_mode_changed rows
      recallMetrics: false,  // wired in PART11; commands.rs:614 returns measured constants from baseline toml
      languageStatus: false, // wired in Pass D; SsbViewer::latest_action_of_type reads SSB
      // E-17 SP4: producers shipped in E-17 SP2 (knowledge::pipeline /
      // deep / hyde) — surface is fully wired. Empty arrays mean
      // "steady state, no events" not "unwired".
      deepPoolCapped: false,
      rrfQualityGate: false,
      hydeSkippedCodeQuery: false,
      rrfComponentWeights: false,
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
    /** E-17 SP4: explicit refresh trigger for the retry button on the
     *  RAG quality cards. Does NOT touch the refcount or polling timer. */
    refresh,
  };
}
export const knowledgeData = createKnowledge();

// ------------------------------------------------------------------
// REASONING drawer
// ------------------------------------------------------------------
function createReasoning() {
  let grcBudgets = $state<GrcBudget[]>([]);
  let groundingSweep = $state<GroundingSweep>({ grounded: 0, verify_needed: 0, ungrounded: 0, claims: [] });
  let csqPending = $state<CsqEntry[]>([]);
  let ggSignals = $state<GgSignals>({ mean_logprob: 0, min_logprob: 0, entropy_mean: 0, entropy_max: 0, token_count: 0 });
  let mistakePreview = $state<MistakeEntry[]>([]);
  let feasibility = $state<{
    overall: 'PASS' | 'FAIL' | 'BLOCKED';
    results: {
      id: string;
      check: string;
      outcome: 'PASS' | 'FAIL' | 'BLOCKED';
      detail: string | null;
      ts: number;
    }[];
    probed_at: number;
  }>({
    overall: 'PASS',
    results: [],
    probed_at: 0,
  });
  let reflectState = $state<ReflectState>({ enabled: false, last_triage: null, last_verdict: null, last_summary: null });
  // F-8 / Slice A5 — held-pending SCF symbolic-conflict rows (newest-first).
  // Empty in steady state; rows appear only when a self-mod config variant
  // contradicts a locked-key invariant. WARN-then-escalate, not rejections.
  let symbolicConflicts = $state<SymbolicConflict[]>([]);

  // F-11 — config-DGM admit-loop rows. Empty in steady state; rows appear only
  // after a `darwin-loop` run with ELAI_DARWINIAN_LOOP_ENABLE=1 + feature
  // darwinian-evolve. Admitted / RejectedByGuard / LoopClosed.
  let darwinianAdmits = $state<DarwinianAdmit[]>([]);

  // H-46 — CMP parent-selection rows. Empty in steady state; rows appear only
  // after a `darwin-loop` run with ELAI_DARWINIAN_LOOP_ENABLE=1 + feature
  // darwinian-evolve. CladeMetaproductivity / ImmediateScoreFallback.
  let cladeSelections = $state<CladeSelection[]>([]);

  // B-5 GAP-1 — anti-rubber-stamp rejection rows. Empty in steady state
  // (no auditor rubber-stamp rejections have fired yet). The
  // ReasoningDrawer renders "Auditor: N rubber stamps rejected this
  // session" from the array length.
  let auditorRubberStampsRejected = $state<AuditorRubberStampRejectionRow[]>([]);

  async function refresh() {
    try {
      const [budgets, sweep, csqs, gg, mistakes, feas, reflect, conflicts, admits, clades, rejections] = await Promise.all([
        stub.get_grc_budgets(),
        stub.get_grounding_sweep(),
        stub.get_csq_pending(),
        stub.get_gg_signals(),
        stub.get_mistake_book(),
        stub.get_feasibility_report(),
        stub.get_reflect_state(),
        stub.get_symbolic_conflicts(),
        stub.get_darwinian_admits(),
        stub.get_clade_selections(),
        stub.get_auditor_rubber_stamp_rejections(),
      ]);
      grcBudgets = budgets;
      groundingSweep = sweep;
      csqPending = csqs;
      ggSignals = gg;
      mistakePreview = mistakes.slice(0, 3);
      auditorRubberStampsRejected = rejections;
      feasibility = {
        overall: (feas.overall as 'PASS' | 'FAIL' | 'BLOCKED') ?? 'PASS',
        results: (feas.results ?? []).map((r) => ({
          id: r.id ?? '',
          check: r.check ?? '',
          outcome: r.outcome ?? 'PASS',
          detail: r.detail ?? null,
          ts: r.ts ?? 0,
        })),
        probed_at: feas.probed_at ?? 0,
      };
      reflectState = reflect;
      symbolicConflicts = conflicts;
      darwinianAdmits = admits;
      cladeSelections = clades;
    } catch (err) {
      console.debug('[reasoningData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get grcBudgets() { return grcBudgets; },
    get groundingSweep() { return groundingSweep; },
    get csqPending() { return csqPending; },
    get ggSignals() { return ggSignals; },
    get mistakePreview() { return mistakePreview; },
    get feasibility() { return feasibility; },
    get reflectState() { return reflectState; },
    // F-8 / Slice A5 — held-pending SCF symbolic-conflict rows.
    get symbolicConflicts() { return symbolicConflicts; },
    // F-11 — config-DGM admit-loop rows (Admitted / RejectedByGuard / LoopClosed).
    get darwinianAdmits() { return darwinianAdmits; },
    // H-46 — CMP parent-selection rows (CladeMetaproductivity / ImmediateScoreFallback).
    get cladeSelections() { return cladeSelections; },
    // B-5 GAP-1 — anti-rubber-stamp rejection rows. Empty in steady state.
    get auditorRubberStampsRejected() { return auditorRubberStampsRejected; },
    unwired: {
      groundingSweep: false, // C-1: get_grounding_sweep reads SSB grounding_sweep_result rows
      csqPending: false,     // C-1: get_csq_pending reads SSB csq_evaluated rows where blocked=true
      feasibility: false,    // B-21 4F: get_feasibility_report decodes real SSB rows
      reflect: false,        // B-39: get_reflect_state wired to Tauri command
      // F-8 / Slice A5: get_symbolic_conflicts reads SSB
      // symbolic_conflict_detected rows. Empty = steady state (no conflicts),
      // not unwired — the SCF detector producer ships in F-8.
      symbolicConflicts: false,
      // F-11: get_darwinian_admits reads SSB darwinian_admit rows. Empty =
      // steady state (loop not run / gated OFF), not unwired — the producer
      // (darwin-loop CLI) ships in F-11 behind feature + ELAI_DARWINIAN_LOOP_ENABLE.
      darwinianAdmits: false,
      // H-46: get_clade_selections reads SSB clade_selected rows. Empty =
      // steady state (loop not run / gated OFF), not unwired — the producer
      // ships in H-46 behind darwinian-evolve feature + ELAI_DARWINIAN_LOOP_ENABLE.
      cladeSelections: false,
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const reasoningData = createReasoning();

// ------------------------------------------------------------------
// ORCHESTRATION drawer
// ------------------------------------------------------------------
function createOrchestration() {
  let fsmSnapshot = $state<FsmSnapshot>({ state: 'IDLE', current_step_id: '', progress: 0, history: [] });
  let evidenceGates = $state<EvidenceGate[]>([]);
  let agents = $state<AgentEntry[]>([]);
  let supervisionLog = $state<SupervisionEvent[]>([]);
  let bspSupersteps = $state<BspSuperstep[]>([]);
  let contextBudget = $state<ContextBudget>({ slots: [], total_used: 0, total_cap: 0 });
  let sessionBranches = $state<SessionBranch[]>([]);
  let antiSpiral = $state<AntiSpiralCounter>({ step_id: '', edit_count: 0, threshold: 3, triggered: false, last_hash: '' });
  let heartbeats = $state<HeartbeatEntry[]>([]);

  async function refresh() {
    try {
      const [snap, gates, ags, supLog, bsp, ctx, branches, spiral, hbs] = await Promise.all([
        stub.get_fsm_state(),
        stub.get_evidence_gates(),
        stub.get_agents(),
        stub.get_supervision_log(),
        stub.get_bsp_supersteps(),
        stub.get_context_budget(),
        stub.get_session_branches(),
        stub.get_anti_spiral_counter(),
        stub.get_heartbeats(),
      ]);
      fsmSnapshot = snap;
      evidenceGates = gates;
      agents = ags;
      supervisionLog = supLog;
      bspSupersteps = bsp;
      contextBudget = ctx;
      sessionBranches = branches;
      antiSpiral = spiral;
      heartbeats = hbs;
    } catch (err) {
      console.debug('[orchestrationData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get fsmSnapshot() { return fsmSnapshot; },
    get evidenceGates() { return evidenceGates; },
    get agents() { return agents; },
    get supervisionLog() { return supervisionLog; },
    get bspSupersteps() { return bspSupersteps.slice(-8); },
    get contextBudget() { return contextBudget; },
    get sessionBranches() { return sessionBranches; },
    get antiSpiral() { return antiSpiral; },
    get heartbeats() { return heartbeats; },
    unwired: {
      antiSpiral: false,  // B-6: wired to latest_anti_spiral_hit SSB row
      heartbeats: true,   // returns [] — no heartbeat SSB producer yet
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const orchestrationData = createOrchestration();

// ------------------------------------------------------------------
// RULES drawer
// ------------------------------------------------------------------
function createRules() {
  let lifecycleCounts = $state<RuleLifecycleCounts>({ PROPOSED: 0, CANDIDATE: 0, SHADOW: 0, ACTIVE: 0, RETIRED: 0 });
  let activeRules = $state<Rule[]>([]);
  let deprecatedPatterns = $state<DeprecatedPattern[]>([]);
  let sourceTiers = $state<SourceTierDistribution[]>([]);
  let lastTransitions = $state<RuleTransitionRow[]>([]);
  let currentFsmState = $state<FsmSnapshot['state']>('IDLE');

  async function refresh() {
    try {
      const fsm = await stub.get_fsm_state();
      currentFsmState = fsm.state;
      // B-3 #11: pull the last 5 rule_transition SSB rows alongside the
      // existing 10s lifecycle refresh. Empty list when no producer has
      // written rows yet — the RulesDrawer renders an explicit
      // "no transitions yet" empty state in that case.
      const [lifecycle, rules, deps, tiers, transitions] = await Promise.all([
        stub.get_rules_lifecycle(),
        stub.get_active_rules(fsm.state),
        stub.get_deprecated_patterns(),
        stub.get_source_quality_tiers(),
        stub.get_rule_transitions(5),
      ]);
      lifecycleCounts = lifecycle;
      activeRules = rules;
      deprecatedPatterns = deps;
      sourceTiers = tiers;
      lastTransitions = transitions;
    } catch (err) {
      console.debug('[rulesData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get lifecycleCounts() { return lifecycleCounts; },
    get activeRules() { return activeRules; },
    get deprecatedPatterns() { return deprecatedPatterns; },
    get sourceTiers() { return sourceTiers; },
    get lastTransitions() { return lastTransitions; },
    get currentFsmState() { return currentFsmState; },
    unwired: {
      sourceTiers: true,  // get_source_quality_tiers returns zeros
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const rulesData = createRules();

// ------------------------------------------------------------------
// RESEARCH drawer
// ------------------------------------------------------------------
//
// Backend payload shapes (verified against commands.rs:2696-2822 and
// web_research_orchestrator.rs:83 WebResearchStepEvent):
//
//   research-progress (one event per iteration step + one on start):
//     {
//       request_id: string,
//       action: "started" | "iterate" | "no_results" | "converged"
//             | "pruned" | "max_iterations",
//       node_id?: number,         // omitted on action="started"
//       query?: string,            // present only on action="started"
//       phi?: number,
//       psi?: number,
//       iteration?: number,
//       sources_fetched?: number,
//       sources_used?: number,
//       fetcher?: string,
//     }
//
//   research-done (separate channel, emitted on run completion or error):
//     ok:    { request_id, status: "ok",    iterations, result_status, sources_used }
//     error: { request_id, status: "error", error }
//
// applyProgress() updates per-node metrics incrementally; on research-done
// it triggers a refresh() to pull the canonical tree from get_research_tree
// since the per-step events do not carry full ResearchTreeNode payloads.
type ResearchProgressAction =
  | 'started'
  | 'iterate'
  | 'no_results'
  | 'converged'
  | 'pruned'
  | 'max_iterations';

interface ResearchProgressPayload {
  request_id: string;
  action: ResearchProgressAction;
  node_id?: number;
  query?: string;
  phi?: number;
  psi?: number;
  iteration?: number;
  sources_fetched?: number;
  sources_used?: number;
  fetcher?: string;
}

type ResearchDonePayload =
  | { request_id: string; status: 'ok'; iterations: number; result_status?: string; sources_used?: number }
  | { request_id: string; status: 'error'; error: string };

const RESEARCH_MAX_NODES = 256;

function createResearch() {
  let researchTree = $state<ResearchTreeNode>({
    id: 'empty', query: 'no active research', stage: 'FORMULATE',
    children: [], source_count: 0, converged: false,
  });
  let sourceTiers = $state<SourceTierDistribution[]>([]);
  let lastProgressError = $state<string | null>(null);
  let activeRequestId = $state<string | null>(null);

  async function refresh() {
    try {
      const [tree, tiers] = await Promise.all([
        stub.get_research_tree(),
        stub.get_source_quality_tiers(),
      ]);
      researchTree = tree;
      sourceTiers = tiers;
    } catch (err) {
      console.debug('[researchData] refresh failed', err);
    }
  }

  function actionToStage(action: ResearchProgressAction): ResearchTreeNode['stage'] {
    if (action === 'started') return 'FORMULATE';
    if (action === 'no_results' || action === 'iterate') return 'SEARCH';
    if (action === 'converged' || action === 'max_iterations' || action === 'pruned') return 'CONVERGE';
    return 'SEARCH';
  }

  function countNodes(root: ResearchTreeNode): number {
    let n = 1;
    for (const c of root.children ?? []) n += countNodes(c);
    return n;
  }

  /** Update a node by id (string match against `node_id` stringified). If not
   *  found, append as a direct child of the root unless RESEARCH_MAX_NODES
   *  would be exceeded — in which case drop the update and warn.
   *  Recursive: supports nested children for future tree-shaped events. */
  function applyNodeMetrics(
    root: ResearchTreeNode,
    nodeId: string,
    payload: ResearchProgressPayload,
  ): ResearchTreeNode {
    if (root.id === nodeId) {
      return {
        ...root,
        stage: actionToStage(payload.action),
        phi: payload.phi ?? root.phi,
        psi: payload.psi ?? root.psi,
        source_count: payload.sources_used ?? root.source_count,
        converged: payload.action === 'converged' ? true : root.converged,
      };
    }
    const children = root.children ?? [];
    let changed = false;
    const nextChildren = children.map((child) => {
      const updated = applyNodeMetrics(child, nodeId, payload);
      if (updated !== child) changed = true;
      return updated;
    });
    if (changed) return { ...root, children: nextChildren };
    if (countNodes(root) >= RESEARCH_MAX_NODES) {
      console.warn('[researchData] node cap reached — dropping update for', nodeId);
      return root;
    }
    return {
      ...root,
      children: [
        ...children,
        {
          id: nodeId,
          query: '(stream)',
          stage: actionToStage(payload.action),
          children: [],
          phi: payload.phi,
          psi: payload.psi,
          source_count: payload.sources_used ?? 0,
          converged: payload.action === 'converged',
        },
      ],
    };
  }

  function applyProgress(p: ResearchProgressPayload) {
    try {
      if (p.action === 'started') {
        activeRequestId = p.request_id;
        researchTree = {
          id: p.request_id,
          query: p.query ?? '(no query)',
          stage: 'FORMULATE',
          children: [],
          source_count: 0,
          converged: false,
        };
        lastProgressError = null;
        return;
      }
      // Per-iteration update — needs a node_id.
      if (p.node_id === undefined) return;
      const nodeId = String(p.node_id);
      researchTree = applyNodeMetrics(researchTree, nodeId, p);
      lastProgressError = null;
    } catch (err) {
      console.warn('[researchData] applyProgress threw', err);
    }
  }

  function applyDone(p: ResearchDonePayload) {
    try {
      if (p.status === 'error') {
        lastProgressError = p.error;
      } else {
        lastProgressError = null;
      }
      // Pull the canonical tree from the backend now that the run finished.
      void refresh().catch((err) => console.warn('[researchData] post-done refresh failed', err));
    } catch (err) {
      console.warn('[researchData] applyDone threw', err);
    }
  }

  // Listener lifecycle — generation counter version (see audit WARN).
  // Each bootstrap/teardown bumps `listenerGen`. A late-resolving listen()
  // compares its captured generation to the current one; mismatch => unlisten
  // immediately. This is robust against bootstrap/teardown/bootstrap races
  // that a boolean `cancelled` flag cannot handle.
  let progressUnlisten: UnlistenFn | null = null;
  let doneUnlisten: UnlistenFn | null = null;
  // B-36 STEP 5: canonical SSE event name `web_research_step` (WIRING_CONTRACT
  // produces_symbols). The backend emits this alongside the legacy
  // `research-progress` alias; we listen to both so the drawer updates
  // regardless of which event arrives first.
  let webResearchStepUnlisten: UnlistenFn | null = null;
  let listenerGen = 0;

  async function startProgressListeners() {
    if (progressUnlisten !== null && doneUnlisten !== null && webResearchStepUnlisten !== null) return;
    listenerGen += 1;
    const myGen = listenerGen;
    try {
      const [un1, un2, un3] = await Promise.all([
        listen<ResearchProgressPayload>('research-progress', (event) => applyProgress(event.payload)),
        listen<ResearchDonePayload>('research-done', (event) => applyDone(event.payload)),
        listen<ResearchProgressPayload>('web_research_step', (event) => applyProgress(event.payload)),
      ]);
      if (myGen !== listenerGen) {
        // Generation rolled — teardown ran (and possibly another bootstrap).
        // Unlisten the listeners we just registered; the active gen owns its own.
        try { un1(); } catch (err) { console.warn('[researchData] late progress unlisten failed', err); }
        try { un2(); } catch (err) { console.warn('[researchData] late done unlisten failed', err); }
        try { un3(); } catch (err) { console.warn('[researchData] late web_research_step unlisten failed', err); }
        return;
      }
      progressUnlisten = un1;
      doneUnlisten = un2;
      webResearchStepUnlisten = un3;
    } catch (err) {
      // Degrade gracefully — drawer continues to poll every 10s.
      console.warn('[researchData] research event listen() failed', err);
    }
  }

  function stopProgressListeners() {
    listenerGen += 1;
    if (progressUnlisten) {
      try { progressUnlisten(); } catch (err) { console.warn('[researchData] progress unlisten failed', err); }
      progressUnlisten = null;
    }
    if (doneUnlisten) {
      try { doneUnlisten(); } catch (err) { console.warn('[researchData] done unlisten failed', err); }
      doneUnlisten = null;
    }
    if (webResearchStepUnlisten) {
      try { webResearchStepUnlisten(); } catch (err) { console.warn('[researchData] web_research_step unlisten failed', err); }
      webResearchStepUnlisten = null;
    }
  }

  // Wrap rc bootstrap/teardown so the listener lifecycle matches the
  // refcount on the polling refresh — both off when no drawer mounted.
  const rc = makeRefCounted(refresh);
  let listenerRefcount = 0;

  return {
    get researchTree() { return researchTree; },
    get sourceTiers() { return sourceTiers; },
    get lastProgressError() { return lastProgressError; },
    get activeRequestId() { return activeRequestId; },
    unwired: {
      sourceTiers: true,
      webBudget: true,        // synthetic placeholder in drawer
      convergence: true,      // phi/psi computed only when research actually runs
    },
    bootstrap() {
      rc.bootstrap();
      listenerRefcount += 1;
      if (listenerRefcount === 1) {
        void startProgressListeners();
      }
    },
    teardown() {
      rc.teardown();
      listenerRefcount = Math.max(0, listenerRefcount - 1);
      if (listenerRefcount === 0) {
        stopProgressListeners();
      }
    },
  };
}
export const researchData = createResearch();

// ------------------------------------------------------------------
// PROVIDERS drawer
// ------------------------------------------------------------------
function createProviders() {
  let vramGauges = $state<VramGauge[]>([]);
  let sidecarStatus = $state<SidecarStatus>({
    loaded: false, model: 'unavailable', fa2: false, dola: false, steering: false,
    turboquant: 'OFF', tokens_per_sec: 0, latency_p50_ms: 0, latency_p99_ms: 0,
    context_window: 0, mcp_bridge_active: false,
  });
  let inferenceMetrics = $state<InferenceMetrics>({
    tokens_per_sec_last_60s: 0, latency_p50_ms: 0, latency_p99_ms: 0,
    total_tokens_session: 0, cloud_fallback_count: 0,
  });
  let mcpStatus = $state<McpStatus>({
    enabled: false, bridge_url: '', connected: false, tools_available: [],
  });

  async function refresh() {
    try {
      const [vram, sidecar, infer, mcp] = await Promise.all([
        stub.get_vram(),
        stub.get_sidecar_status(),
        stub.get_inference_metrics(),
        stub.get_mcp_status(),
      ]);
      vramGauges = vram;
      sidecarStatus = sidecar;
      inferenceMetrics = infer;
      mcpStatus = mcp;
    } catch (err) {
      console.debug('[providersData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get vramGauges() { return vramGauges; },
    get sidecarStatus() { return sidecarStatus; },
    get inferenceMetrics() { return inferenceMetrics; },
    get mcpStatus() { return mcpStatus; },
    unwired: {
      inferenceMetrics: true,  // get_inference_metrics returns zeros
      modelPool: true,         // synthesised in drawer; no real ModelPool query
      tokensPerSec: true,      // sparkline is synthetic; no per-second history
      colocation: true,        // GPU multiplexing event log not emitted
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const providersData = createProviders();

// ------------------------------------------------------------------
// PROJECT-INTELLIGENCE drawer
// ------------------------------------------------------------------
function createProjectIntelligence() {
  let profile = $state<ProjectProfile>({
    tier1: { language: '', build_tool: '', framework: '', test_runner: '', ci: '', linter: '' },
    tier2: { conventions: [] },
    tier3: { architecture_inference: '', telescope_summaries: [] },
  });

  async function refresh() {
    try {
      profile = await stub.get_project_intelligence();
    } catch (err) {
      console.debug('[projectIntelligenceData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get profile() { return profile; },
    unwired: {
      override: true,  // save_override is logStub-only; no persistence command yet
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const projectIntelligenceData = createProjectIntelligence();

// ------------------------------------------------------------------
// ANTI-STALENESS drawer
// ------------------------------------------------------------------
function createAntiStaleness() {
  let deprecatedPatterns = $state<DeprecatedPattern[]>([]);
  let depAudit = $state<DepAuditResult[]>([]);
  let baselineResults = $state<BaselineResults>({ metrics: [], run_ts: 0, deterministic: false });
  let antiSpiral = $state<AntiSpiralCounter>({ step_id: '', edit_count: 0, threshold: 3, triggered: false, last_hash: '' });

  async function refresh() {
    try {
      const [deps, audit, baseline, spiral] = await Promise.all([
        stub.get_deprecated_patterns(),
        stub.get_dep_audit_results(),
        stub.get_baseline_results(),
        stub.get_anti_spiral_counter(),
      ]);
      deprecatedPatterns = deps;
      depAudit = audit;
      baselineResults = baseline;
      antiSpiral = spiral;
    } catch (err) {
      console.debug('[antiStalenessData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get deprecatedPatterns() { return deprecatedPatterns; },
    get depAudit() { return depAudit; },
    get baselineResults() { return baselineResults; },
    get antiSpiral() { return antiSpiral; },
    unwired: {
      depAudit: true,         // returns []
      baselineResults: false, // wired in PART11; commands.rs:1757 returns measured metrics from phase_A1_baseline.toml
      antiSpiral: false,  // B-6: wired to latest_anti_spiral_hit SSB row via get_anti_spiral_counter
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const antiStalenessData = createAntiStaleness();

// ------------------------------------------------------------------
// TOOLS-HOOKS drawer
// ------------------------------------------------------------------
function createToolsHooks() {
  let hookExecutions = $state<HookExecution[]>([]);
  let shortcoderStats = $state<ShortCoderStats>({
    savings_pct: 0, exact_hits: 0, whitespace_hits: 0, ellipsis_hits: 0, total_edits: 0,
  });
  let failureTaxonomy = $state<FailureTaxonomyResult | null>(null);
  let hiddenToolNames = $state<string[]>([]);

  async function refresh() {
    try {
      const [hooks, sc, taxonomy, toolVis] = await Promise.all([
        stub.get_hook_executions(),
        stub.get_shortcoder_stats(),
        stub.get_failure_taxonomy_last(),
        stub.get_tool_visibility(),
      ]);
      hookExecutions = hooks;
      shortcoderStats = sc;
      failureTaxonomy = taxonomy;
      hiddenToolNames = toolVis.hidden_tools;
    } catch (err) {
      console.debug('[toolsHooksData] refresh failed', err);
    }
  }

  const rc = makeRefCounted(refresh);

  return {
    get hookExecutions() { return hookExecutions; },
    get shortcoderStats() { return shortcoderStats; },
    get failureTaxonomy() { return failureTaxonomy; },
    get hiddenTools() { return hiddenToolNames; },
    unwired: {
      tieredAnalysis: true,  // synthetic per-tier counts in drawer
      shortcoderRules: true, // per-rule hit counts are synthetic
      taxonomyDistribution: true, // per-category counts are synthetic
      toolFilter: false,     // wired in PART11; commands.rs:3303 returns MODEL_HIDDEN_TOOL_NAMES
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
  };
}
export const toolsHooksData = createToolsHooks();

// ------------------------------------------------------------------
// WEB-QUALITY drawer
// ------------------------------------------------------------------
function createWebQuality() {
  let buildSlots = $state<BuildSlot[]>([]);
  let devServers = $state<DevServer[]>([]);
  let webAragState = $state<WebAragState>({
    index_size: 0, last_crawled_ts: 0, pages_indexed: 0, quality_score: 0,
  });
  let sourceTiers = $state<SourceTierDistribution[]>([]);
  // Phase A (s-69820): live `list_scaffold_templates` Tauri command. Falls back
  // to mock SCAFFOLD_TEMPLATES when running outside Tauri (browser-only dev).
  let scaffoldTemplates = $state<ScaffoldTemplate[]>([]);
  let scaffoldTemplatesIsLive = $state(false);
  // Web rules counts: backend command returns lifecycle counts but not domain-tagged
  // T1/T2/T3 split. Display zeros until the backend exposes a domain filter.
  const webRulesCounts = { t1: 0, t2: 0, t3: 0 };

  // Phase C1+C2 (s-67812) — lint + Lighthouse + Chromium-probe state.
  // refresh() does NOT poll these; lint runs spawn npx subprocesses and
  // Lighthouse takes 25-45s, so they are user-action driven. The C2
  // action handlers below populate these fields; resetWebQualityResults()
  // clears them on explicit user request.
  let lintFindings = $state<LintFinding[]>([]);
  let lighthouseReport = $state<LighthouseReport | null>(null);
  let chromiumAvailable = $state(false);
  let lintRunning = $state(false);
  let lighthouseRunning = $state(false);
  let lintHasRun = $state(false);
  let lighthouseHasRun = $state(false);
  let chromiumProbeDone = $state(false);
  // Plain (non-reactive) re-entry guard for probeChromium. Refresh() may
  // be called concurrently (manual refresh during a polling tick); without
  // this gate two probes can fire if both observe !chromiumProbeDone.
  let chromiumProbeInFlight = false;
  let lintLastError = $state<string | null>(null);
  let lighthouseLastError = $state<string | null>(null);

  async function refresh() {
    try {
      const [slots, servers, arag, tiers, templates] = await Promise.all([
        stub.get_build_slots(),
        stub.get_dev_servers(),
        stub.get_web_arag_state(),
        stub.get_source_quality_tiers(),
        stub.list_scaffold_templates(),
      ]);
      buildSlots = slots;
      devServers = servers;
      webAragState = arag;
      sourceTiers = tiers;
      scaffoldTemplates = templates;
      // Live when at least one template carries the new sections field
      // (mock data leaves sections undefined).
      scaffoldTemplatesIsLive = templates.some((t) => Array.isArray(t.sections));
      // Phase C2 (s-67812): one-shot Chromium probe on first refresh.
      // Chromium availability is environmental; polling it every 10s is
      // wasteful, so we gate behind chromiumProbeDone.
      if (!chromiumProbeDone) {
        await probeChromium();
      }
    } catch (err) {
      console.debug('[webQualityData] refresh failed', err);
    }
  }

  // Phase C2 (s-67812) — action handlers. Each handler short-circuits if
  // its corresponding *Running flag is set, so concurrent invocations are
  // a no-op rather than a queue. Errors are surfaced via *LastError fields
  // (typed string | null) — never silently swallowed.
  async function runWebLint(
    projectDir: string,
    eslint: boolean,
    stylelint: boolean,
    axe: boolean,
  ): Promise<void> {
    if (lintRunning) return;
    if (!projectDir.trim()) {
      lintLastError = 'project_dir is required';
      return;
    }
    if (!eslint && !stylelint && !axe) {
      lintLastError = 'at least one tool must be enabled';
      return;
    }
    lintRunning = true;
    lintLastError = null;
    try {
      const findings = await stub.run_web_lint_check(projectDir, eslint, stylelint, axe);
      lintFindings = findings;
      lintHasRun = true;
    } catch (err) {
      lintLastError = err instanceof Error ? err.message : String(err);
      console.error('[webQualityData] runWebLint failed', err);
    } finally {
      lintRunning = false;
    }
  }

  async function runLighthouseAudit(projectDir: string, url: string): Promise<void> {
    if (lighthouseRunning) return;
    if (!projectDir.trim() || !url.trim()) {
      lighthouseLastError = 'project_dir and url are required';
      return;
    }
    lighthouseRunning = true;
    lighthouseLastError = null;
    try {
      const report = await stub.run_lighthouse_audit(projectDir, url);
      lighthouseReport = report;
      lighthouseHasRun = true;
    } catch (err) {
      lighthouseLastError = err instanceof Error ? err.message : String(err);
      console.error('[webQualityData] runLighthouseAudit failed', err);
    } finally {
      lighthouseRunning = false;
    }
  }

  async function probeChromium(): Promise<void> {
    if (chromiumProbeInFlight) return;
    chromiumProbeInFlight = true;
    try {
      chromiumAvailable = await stub.probe_chromium_runtime();
    } catch (err) {
      console.debug('[webQualityData] probeChromium failed (likely no Tauri)', err);
      chromiumAvailable = false;
    } finally {
      chromiumProbeDone = true;
      chromiumProbeInFlight = false;
    }
  }

  function resetWebQualityResults(): void {
    lintFindings = [];
    lighthouseReport = null;
    lintHasRun = false;
    lighthouseHasRun = false;
    lintLastError = null;
    lighthouseLastError = null;
    // Intentionally NOT resetting chromiumAvailable / chromiumProbeDone:
    // those reflect the environment, not result data.
  }

  const rc = makeRefCounted(refresh);

  return {
    get buildSlots() { return buildSlots; },
    get devServers() { return devServers; },
    get webAragState() { return webAragState; },
    get sourceTiers() { return sourceTiers; },
    get scaffoldTemplates() { return scaffoldTemplates; },
    get webRulesCounts() { return webRulesCounts; },
    get lintFindings() { return lintFindings; },
    get lighthouseReport() { return lighthouseReport; },
    get chromiumAvailable() { return chromiumAvailable; },
    get lintRunning() { return lintRunning; },
    get lighthouseRunning() { return lighthouseRunning; },
    get lintHasRun() { return lintHasRun; },
    get lighthouseHasRun() { return lighthouseHasRun; },
    get chromiumProbeDone() { return chromiumProbeDone; },
    get lintLastError() { return lintLastError; },
    get lighthouseLastError() { return lighthouseLastError; },
    unwired: {
      buildSlots: true,        // get_build_slots is partial; producer not actuated
      get devServers() {
        // Live as soon as any registered dev-server appears in the list;
        // get_dev_servers is real Tauri-backed in Phase A.
        return devServers.length === 0;
      },
      get scaffoldTemplates() {
        // Phase A: live whenever the live-shape detector matched at least
        // one entry in the last refresh.
        return !scaffoldTemplatesIsLive;
      },
      webAragState: true,      // returns zeros
      webRulesCounts: true,    // T1/T2/T3 split not exposed
      sourceTiers: true,
      // Phase C2 (s-67812): live as soon as the user has triggered a run.
      // Before the first run, the section displays its empty state but is
      // not "unwired" — the action handlers exist, only no result yet.
      get lintFindings() { return !lintHasRun; },
      get lighthouseReport() { return !lighthouseHasRun; },
      get chromiumAvailable() { return !chromiumProbeDone; },
    },
    bootstrap: rc.bootstrap,
    teardown: rc.teardown,
    /// Phase A audit fix (s-69820): standalone one-shot refresh that does
    /// NOT touch the refcount or the polling timer. Call this from action
    /// handlers (Use template / Stop server / etc.) after a mutation;
    /// `bootstrap()` would leak a refcount on every click.
    refresh,
    // Phase C2 (s-67812) — user-action handlers.
    runWebLint,
    runLighthouseAudit,
    probeChromium,
    resetWebQualityResults,
  };
}
export const webQualityData = createWebQuality();
