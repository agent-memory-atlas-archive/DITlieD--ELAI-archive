// api/mock-orbit.ts
// Three orbit fixtures: small single-concept, multi-concept constellation,
// mid-research-tree. Used for development and visual tests until the orbit
// is wired to live commands.rs/Tauri data.
//
// Field shapes mirror Rust types verbatim where possible:
// - ConceptDecomposition / ConceptNode / PhaseNode / StepNode / BabyStepSpec
//   from crates/orchestrator/src/research_tree.rs:1245-1502
// - ResearchTreeNode lifecycle from crates/orchestrator/src/web_research_orchestrator.rs
// - SourceTier from crates/web_search/src/source_quality.rs
// - PageRank shape from crates/knowledge/src/pagerank/mod.rs

import type {
  OrbitFixture,
  ConceptStub,
  PhaseStub,
  StepStub,
  BabyStepStub,
  PageRankNodeStub,
  DkgEdgeStub,
} from '../orbit/types';
import type { EvidenceGate, ResearchTreeNode, ContextBudgetSlot, GrcBudget } from './types';

// -----------------------------------------------------------------
// Fixture 1 — single concept, small task. The "easy" case.
// -----------------------------------------------------------------
const F1_BABIES_S1: BabyStepStub[] = [
  { id: 'bs-1-1', label: 'Sketch toggle component', task: 'Initial component shape with no styling', estimated_tokens: 600 },
  { id: 'bs-1-2', label: 'Wire toggle to Svelte store', task: 'Connect toggle state to a $state store', estimated_tokens: 500 },
  { id: 'bs-1-3', label: 'Apply theme tokens',         task: 'Bind --bg-base / --text-primary etc to theme', estimated_tokens: 700 },
];

const F1_BABIES_S2: BabyStepStub[] = [
  { id: 'bs-2-1', label: 'Write toggle.spec.ts',  task: 'Vitest happy path + a11y assertions', estimated_tokens: 500 },
  { id: 'bs-2-2', label: 'Verify keyboard',       task: 'Tab + space + enter activations',   estimated_tokens: 400 },
];

const F1_PHASES: PhaseStub[] = [
  {
    id: 'phase-1',
    label: 'Component skeleton',
    steps: [
      { id: 'step-1-1', label: 'Component + binding', baby_steps: F1_BABIES_S1 },
    ],
  },
  {
    id: 'phase-2',
    label: 'Acceptance tests',
    depends_on: ['concept-1'],
    steps: [
      { id: 'step-2-1', label: 'Spec + manual check', baby_steps: F1_BABIES_S2 },
    ],
  },
];

const F1_CONCEPTS: ConceptStub[] = [
  { id: 'concept-1', name: 'Dark Mode Toggle', description: 'Settings-screen toggle', phases: F1_PHASES },
];

const F1_GATES: EvidenceGate[] = [
  { id: 'g1', name: 'KnowledgeReadiness', verdict: 'pass', checked_at: 1000 },
  { id: 'g2', name: 'GroundingSweep',     verdict: 'pass', checked_at: 1100 },
  { id: 'g3', name: 'TestFailReasons',    verdict: 'pending' },
  { id: 'g4', name: 'TestFlipped',        verdict: 'pending' },
  { id: 'g5', name: 'ReplanFastTrack',    verdict: 'pending' },
  { id: 'g6', name: 'ZeroBlocks',         verdict: 'pending' },
  { id: 'g7', name: 'DkgChangelog',       verdict: 'pending' },
];

const F1_CTX_BUDGET: ContextBudgetSlot[] = [
  { name: 'SystemPrompt',  used_tokens: 480,   total_tokens: 500,   pct: 96 },
  { name: 'Rules',         used_tokens: 1200,  total_tokens: 2000,  pct: 60 },
  { name: 'PhaseSpec',     used_tokens: 800,   total_tokens: 1500,  pct: 53 },
  { name: 'RepoMap',       used_tokens: 2400,  total_tokens: 4000,  pct: 60 },
  { name: 'CodeContext',   used_tokens: 6800,  total_tokens: 12000, pct: 57 },
  { name: 'SsbEntries',    used_tokens: 1100,  total_tokens: 2000,  pct: 55 },
  { name: 'GrcHistory',    used_tokens: 750,   total_tokens: 1500,  pct: 50 },
  { name: 'OutputReserve', used_tokens: 0,     total_tokens: 2500,  pct: 0 },
];

const F1_AGENT_BUDGETS: GrcBudget[] = [
  { role: 'executor',         calls_used: 2,  total_calls: 8,  calls_remaining: 6,  chars_used: 600,  total_chars: 12000, chars_remaining: 11400 },
  { role: 'auditor',          calls_used: 0,  total_calls: 10, calls_remaining: 10, chars_used: 0,    total_chars: 12000, chars_remaining: 12000 },
  { role: 'researcher',       calls_used: 0,  total_calls: 6,  calls_remaining: 6,  chars_used: 0,    total_chars: 9000,  chars_remaining: 9000 },
  { role: 'designer',         calls_used: 0,  total_calls: 5,  calls_remaining: 5,  chars_used: 0,    total_chars: 7500,  chars_remaining: 7500 },
  { role: 'prompt-engineer',  calls_used: 0,  total_calls: 4,  calls_remaining: 4,  chars_used: 0,    total_chars: 6000,  chars_remaining: 6000 },
  { role: 'context-compiler', calls_used: 0,  total_calls: 4,  calls_remaining: 4,  chars_used: 0,    total_chars: 6000,  chars_remaining: 6000 },
  { role: 'memory-writer',    calls_used: 0,  total_calls: 3,  calls_remaining: 3,  chars_used: 0,    total_chars: 4500,  chars_remaining: 4500 },
  { role: 'media-validator',  calls_used: 0,  total_calls: 3,  calls_remaining: 3,  chars_used: 0,    total_chars: 4500,  chars_remaining: 4500 },
];

export const ORBIT_FIXTURE_SMALL: OrbitFixture = {
  task_id: 'task-toggle-001',
  idea: 'Add dark mode toggle to settings',
  fsm: 'EXECUTING',
  active_concept_id: 'concept-1',
  active_phase_id: 'phase-1',
  active_step_id: 'step-1-1',
  active_baby_step_id: 'bs-1-2',
  grc_step: 'act',
  active_agent: 'executor',
  concepts: F1_CONCEPTS,
  evidence_gates: F1_GATES,
  context_budget: F1_CTX_BUDGET,
  agent_budgets: F1_AGENT_BUDGETS,
};

// -----------------------------------------------------------------
// Fixture 2 — multi-concept constellation. "Build Redis pool with monitoring".
// Two concepts: connection pool, health monitoring. Constellation layout
// at concept-zoom; pannable.
// -----------------------------------------------------------------
const F2_C1_PHASES: PhaseStub[] = [
  {
    id: 'phase-c1-research',
    label: 'Research pool libs',
    steps: [
      {
        id: 'step-c1-r1',
        label: 'Compare bb8 / deadpool / r2d2',
        baby_steps: [
          { id: 'bs-c1-r1-1', label: 'Read bb8 docs',           task: 'docs.rs/bb8 main page',                  estimated_tokens: 400 },
          { id: 'bs-c1-r1-2', label: 'Read deadpool docs',      task: 'github deadpool README',                 estimated_tokens: 400 },
          { id: 'bs-c1-r1-3', label: 'Compare ergonomics',      task: 'GRC verify against project rules',       estimated_tokens: 500 },
        ],
      },
      {
        id: 'step-c1-r2',
        label: 'Bench representative throughput',
        baby_steps: [
          { id: 'bs-c1-r2-1', label: 'Set up criterion harness', task: 'cargo bench scaffolding',               estimated_tokens: 600 },
          { id: 'bs-c1-r2-2', label: 'Measure 1k connections',   task: 'simulate concurrent saturation',        estimated_tokens: 700 },
        ],
      },
    ],
  },
  {
    id: 'phase-c1-design',
    label: 'Pool API design',
    depends_on: [],
    steps: [
      {
        id: 'step-c1-d1',
        label: 'Public surface',
        baby_steps: [
          { id: 'bs-c1-d1-1', label: 'Trait sketch',         task: 'PoolHandle + RetryPolicy',                  estimated_tokens: 500 },
          { id: 'bs-c1-d1-2', label: 'Builder pattern',      task: 'with_max / with_timeout / with_retries',    estimated_tokens: 500 },
        ],
      },
    ],
  },
  {
    id: 'phase-c1-impl',
    label: 'Implement pool',
    depends_on: [],
    steps: [
      {
        id: 'step-c1-i1',
        label: 'Add Cargo.toml dep',
        baby_steps: [{ id: 'bs-c1-i1-1', label: 'Add bb8 dependency',     task: 'cargo add bb8 with features',  estimated_tokens: 300 }],
      },
      {
        id: 'step-c1-i2',
        label: 'Pool builder',
        baby_steps: [
          { id: 'bs-c1-i2-1', label: 'PoolBuilder type',     task: 'public struct + with_*() chain',            estimated_tokens: 800 },
          { id: 'bs-c1-i2-2', label: 'build() returns Result', task: 'no .unwrap; thiserror',                   estimated_tokens: 600 },
        ],
      },
      {
        id: 'step-c1-i3',
        label: 'Connection lifecycle',
        baby_steps: [
          { id: 'bs-c1-i3-1', label: 'Idle reaper',          task: 'background tokio task',                     estimated_tokens: 700 },
          { id: 'bs-c1-i3-2', label: 'Health check',          task: 'PING command on checkout',                  estimated_tokens: 500 },
        ],
      },
    ],
  },
  {
    id: 'phase-c1-test',
    label: 'Tests',
    depends_on: [],
    steps: [
      {
        id: 'step-c1-t1',
        label: 'Unit',
        baby_steps: [
          { id: 'bs-c1-t1-1', label: 'Builder unit test',    task: 'all with_* combinations',                   estimated_tokens: 400 },
          { id: 'bs-c1-t1-2', label: 'Lifecycle unit test',  task: 'mock connection + reaper',                  estimated_tokens: 600 },
        ],
      },
      {
        id: 'step-c1-t2',
        label: 'Integration',
        baby_steps: [
          { id: 'bs-c1-t2-1', label: 'Real Redis container',  task: 'testcontainers-rs',                         estimated_tokens: 700 },
        ],
      },
    ],
  },
];

const F2_C2_PHASES: PhaseStub[] = [
  {
    id: 'phase-c2-design',
    label: 'Health endpoint shape',
    depends_on: ['concept-pool'],
    steps: [
      {
        id: 'step-c2-d1',
        label: 'Define metrics',
        baby_steps: [
          { id: 'bs-c2-d1-1', label: 'Active vs idle counters', task: 'mpsc channel sizes',           estimated_tokens: 500 },
          { id: 'bs-c2-d1-2', label: 'Latency p50/p99',         task: 'rolling window over last 60s', estimated_tokens: 600 },
        ],
      },
      {
        id: 'step-c2-d2',
        label: 'Endpoint contract',
        baby_steps: [
          { id: 'bs-c2-d2-1', label: '/health JSON schema',     task: 'serde struct + axum route',    estimated_tokens: 500 },
        ],
      },
    ],
  },
  {
    id: 'phase-c2-impl',
    label: 'Wire metrics',
    depends_on: ['concept-pool'],
    steps: [
      {
        id: 'step-c2-i1',
        label: 'Counters',
        baby_steps: [
          { id: 'bs-c2-i1-1', label: 'AtomicU64 active count',  task: 'increment on checkout',        estimated_tokens: 400 },
          { id: 'bs-c2-i1-2', label: 'AtomicU64 idle count',    task: 'decrement on checkin',         estimated_tokens: 400 },
        ],
      },
      {
        id: 'step-c2-i2',
        label: 'Histogram',
        baby_steps: [
          { id: 'bs-c2-i2-1', label: 'rolling p50/p99',         task: 'hdrhistogram, 60s window',     estimated_tokens: 700 },
        ],
      },
    ],
  },
  {
    id: 'phase-c2-test',
    label: 'Verify',
    depends_on: ['concept-pool'],
    steps: [
      {
        id: 'step-c2-t1',
        label: 'Smoke',
        baby_steps: [
          { id: 'bs-c2-t1-1', label: 'curl /health',            task: 'assert 200 + json shape',      estimated_tokens: 300 },
        ],
      },
    ],
  },
];

const F2_CONCEPTS: ConceptStub[] = [
  { id: 'concept-pool',    name: 'Connection Pool',  description: 'bb8 pool with retries',          phases: F2_C1_PHASES },
  { id: 'concept-monitor', name: 'Health Monitoring', description: 'Active/idle gauges, p50/p99',    phases: F2_C2_PHASES },
];

export const ORBIT_FIXTURE_CONSTELLATION: OrbitFixture = {
  task_id: 'task-redis-pool-002',
  idea: 'Build Redis connection pool with monitoring',
  fsm: 'EXECUTING',
  active_concept_id: 'concept-pool',
  // Active phase placed at index 2 of 4 (≈ bottom of orbit) so the
  // active branch isn't biased to the top — orbit reads as a constellation
  // rather than a vertical string.
  active_phase_id: 'phase-c1-impl',
  active_step_id: 'step-c1-i2',
  active_baby_step_id: 'bs-c1-i2-1',
  grc_step: 'act',
  active_agent: 'executor',
  concepts: F2_CONCEPTS,
  evidence_gates: F1_GATES,
  context_budget: F1_CTX_BUDGET,
  agent_budgets: F1_AGENT_BUDGETS,
};

// -----------------------------------------------------------------
// Fixture 3 — RESEARCHING with a partially-converged research tree.
// FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE node states; phi/psi values
// per crates/orchestrator/src/web_research_orchestrator.rs convergence
// thresholds (phi >= 0.7, psi >= 0.6).
// -----------------------------------------------------------------
const F3_RESEARCH_TREE: ResearchTreeNode = {
  id: 'r-root',
  query: 'How does Anthropic web_search API rate-limit?',
  stage: 'REFLECT',
  source_count: 4,
  converged: false,
  phi: 0.62,
  psi: 0.58,
  children: [
    {
      id: 'r-child-1',
      query: 'requests-per-minute limit on web_search',
      stage: 'CONVERGE',
      source_count: 3,
      converged: true,
      phi: 0.81,
      psi: 0.74,
    },
    {
      id: 'r-child-2',
      query: 'Anthropic batch API + web_search interaction',
      stage: 'EXTRACT',
      source_count: 2,
      converged: false,
      phi: 0.41,
      psi: 0.55,
      children: [
        {
          id: 'r-child-2-a',
          query: 'is web_search exposed in messages.batches?',
          stage: 'SEARCH',
          source_count: 1,
          converged: false,
          phi: 0.20,
          psi: 0.30,
        },
      ],
    },
    {
      id: 'r-child-3',
      query: 'cooldown after 429',
      stage: 'FORMULATE',
      source_count: 0,
      converged: false,
      phi: 0,
      psi: 0,
    },
  ],
};

export const ORBIT_FIXTURE_RESEARCH: OrbitFixture = {
  task_id: 'task-anthropic-rate-limits-003',
  idea: 'Understand Anthropic web_search rate limits',
  fsm: 'RESEARCHING',
  active_concept_id: 'concept-research',
  grc_step: 'verify',
  active_agent: 'researcher',
  concepts: [
    {
      id: 'concept-research',
      name: 'Rate-limit knowledge',
      phases: [
        {
          id: 'phase-r1',
          label: 'Research lifecycle',
          steps: [{ id: 'step-r1', label: 'Active research', baby_steps: [{ id: 'bs-r1', label: 'Reading sources', task: '' }] }],
        },
      ],
    },
  ],
  research_tree: F3_RESEARCH_TREE,
  evidence_gates: F1_GATES,
  context_budget: F1_CTX_BUDGET,
  agent_budgets: F1_AGENT_BUDGETS,
};

// -----------------------------------------------------------------
// Mode 2 — DKG + PageRank stub for "existing project" rendering.
// Modeled on real ELAI symbol distribution: handful of high-rank entry
// points (cli/main.rs, fsm.rs::transition_with_metrics) + medium-rank
// orchestrator helpers + tail of utility nodes.
// -----------------------------------------------------------------
export const ORBIT_PAGERANK_NODES: PageRankNodeStub[] = [
  { id: 'n-fsm',          name: 'orchestrator::fsm::transition_with_metrics', score: 0.182, file: 'crates/orchestrator/src/fsm.rs',                       effect_tags: [] },
  { id: 'n-runner',       name: 'orchestrator::task_runner::run_baby_step',   score: 0.171, file: 'crates/orchestrator/src/task_runner.rs',               effect_tags: ['filesystem'] },
  { id: 'n-evidence',     name: 'orchestrator::evidence::all_gates',          score: 0.144, file: 'crates/orchestrator/src/evidence.rs',                  effect_tags: [] },
  { id: 'n-bsp',          name: 'orchestrator::bsp::engine::run_superstep',   score: 0.118, file: 'crates/orchestrator/src/bsp/engine.rs',                effect_tags: [] },
  { id: 'n-research',     name: 'orchestrator::web_research::reflect',        score: 0.102, file: 'crates/orchestrator/src/web_research_orchestrator.rs', effect_tags: ['network'] },
  { id: 'n-pageranker',   name: 'knowledge::pagerank::compute',               score: 0.091, file: 'crates/knowledge/src/pagerank/mod.rs',                 effect_tags: [] },
  { id: 'n-dkg',          name: 'knowledge::dkg::ingest_file',                score: 0.084, file: 'crates/knowledge/src/dkg.rs',                          effect_tags: ['filesystem', 'database'] },
  { id: 'n-grc',          name: 'reasoning::engine::think',                   score: 0.077, file: 'crates/reasoning/src/engine.rs',                       effect_tags: [] },
  { id: 'n-spiral',       name: 'orchestrator::anti_spiral::record',          score: 0.054, file: 'crates/orchestrator/src/anti_spiral.rs',               effect_tags: [] },
  { id: 'n-budget',       name: 'orchestrator::context_budget::allocate',     score: 0.048, file: 'crates/orchestrator/src/context_budget.rs',            effect_tags: [] },
  { id: 'n-bubble',       name: 'orchestrator::pipeline_tree::bubble_up',     score: 0.041, file: 'crates/orchestrator/src/pipeline_tree_navigator.rs',   effect_tags: [] },
  { id: 'n-source',       name: 'web_search::source_quality::tier',           score: 0.034, file: 'crates/web_search/src/source_quality.rs',              effect_tags: ['network'] },
  { id: 'n-ssb',          name: 'orchestrator::ssb::write_payload',           score: 0.030, file: 'crates/orchestrator/src/ssb.rs',                       effect_tags: ['database'] },
  { id: 'n-cli',          name: 'cli::main',                                  score: 0.025, file: 'crates/cli/src/main.rs',                               effect_tags: ['filesystem', 'network'] },
  { id: 'n-hooks',        name: 'plugins::hooks::dispatch',                   score: 0.022, file: 'crates/plugins/src/hooks.rs',                          effect_tags: [] },
];

export const ORBIT_PAGERANK_EDGES: DkgEdgeStub[] = [
  { source: 'n-cli',        target: 'n-runner',     edge_type: 'call',    weight: 0.9 },
  { source: 'n-runner',     target: 'n-fsm',        edge_type: 'call',    weight: 0.95 },
  { source: 'n-runner',     target: 'n-bsp',        edge_type: 'call',    weight: 0.8 },
  { source: 'n-runner',     target: 'n-grc',        edge_type: 'call',    weight: 0.7 },
  { source: 'n-bsp',        target: 'n-evidence',   edge_type: 'call',    weight: 0.7 },
  { source: 'n-bsp',        target: 'n-budget',     edge_type: 'call',    weight: 0.6 },
  { source: 'n-bsp',        target: 'n-research',   edge_type: 'call',    weight: 0.5 },
  { source: 'n-bsp',        target: 'n-spiral',     edge_type: 'call',    weight: 0.5 },
  { source: 'n-research',   target: 'n-source',     edge_type: 'call',    weight: 0.6 },
  { source: 'n-research',   target: 'n-pageranker', edge_type: 'data_dep', weight: 0.4 },
  { source: 'n-pageranker', target: 'n-dkg',        edge_type: 'call',    weight: 0.7 },
  { source: 'n-runner',     target: 'n-bubble',     edge_type: 'call',    weight: 0.5 },
  { source: 'n-runner',     target: 'n-ssb',        edge_type: 'call',    weight: 0.6 },
  { source: 'n-runner',     target: 'n-hooks',      edge_type: 'call',    weight: 0.5 },
  { source: 'n-fsm',        target: 'n-ssb',        edge_type: 'call',    weight: 0.7 },
];

export const ORBIT_FIXTURE_MAP: OrbitFixture = {
  task_id: 'task-codebase-explore-004',
  idea: 'Explore the orchestrator crate',
  fsm: 'IDLE',
  concepts: [],
  pagerank_nodes: ORBIT_PAGERANK_NODES,
  pagerank_edges: ORBIT_PAGERANK_EDGES,
};
