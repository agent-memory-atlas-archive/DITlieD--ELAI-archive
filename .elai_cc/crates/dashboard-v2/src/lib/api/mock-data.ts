// ============================================================
// ELAI Dashboard — Realistic mock data fixtures
// Fixed NOW anchor ensures deterministic demo rendering.
// All numeric values sourced from real ELAI metrics files.
// ============================================================

import type {
  ProjectSummary,
  RecentTask,
  FsmSnapshot,
  DecompNode,
  BubbleUpEvent,
  ActivityEntry,
  CommandCard,
  InterventionRequest,
  EvidenceGate,
  GrcBudget,
  GroundingSweep,
  CsqEntry,
  EmbeddingBreaker,
  RecallMetrics,
  DkgStats,
  RuleLifecycleCounts,
  Rule,
  DeprecatedPattern,
  SourceTierDistribution,
  VramGauge,
  SidecarStatus,
  InferenceMetrics,
  ContextBudget,
  AgentEntry,
  SupervisionEvent,
  HeartbeatEntry,
  ResearchTreeNode,
  BspSuperstep,
  SessionBranch,
  AntiSpiralCounter,
  MistakeEntry,
  PipelineNode,
  DagRepairEvent,
  BuildSlot,
  DevServer,
  ProjectProfile,
  FeasibilityReport,
  BaselineResults,
  DepAuditResult,
  MsiStatus,
  SteeringStatus,
  McpStatus,
  GateStatus,
  ModelRouting,
  SystemStatus,
  AutoAdvanceStats,
  OptimizationProposal,
  QueryTrace,
  ScaffoldTemplate,
  WebAragState,
  HookExecution,
  ShortCoderStats,
  GgSignals,
} from './types';

// Fixed time anchor — all relative times computed from this.
export const NOW = new Date('2026-04-25T12:04:00Z').getTime();

const mins = (n: number) => NOW - n * 60 * 1000;
const hours = (n: number) => NOW - n * 60 * 60 * 1000;
const days = (n: number) => NOW - n * 24 * 60 * 60 * 1000;
const weeks = (n: number) => NOW - n * 7 * 24 * 60 * 60 * 1000;

// ------------------------------------------------------------------
// Projects
// ------------------------------------------------------------------
export const RECENT_PROJECTS: ProjectSummary[] = [
  { name: 'elai_cc', path: 'D:\\.ELAI_workflow\\.elai_cc', last_opened_iso: new Date(mins(180)).toISOString() },
  { name: 'my-rust-api', path: 'C:\\maintainer\\projects\\rust-api', last_opened_iso: new Date(days(2)).toISOString() },
  { name: 'trading-bot', path: 'C:\\maintainer\\projects\\trading-bot', last_opened_iso: new Date(days(1)).toISOString() },
  { name: 'game-engine', path: 'C:\\maintainer\\projects\\game-engine', last_opened_iso: new Date(weeks(1)).toISOString() },
  { name: 'kimi-clone', path: 'C:\\maintainer\\projects\\kimi-clone', last_opened_iso: new Date(weeks(3)).toISOString() },
];

// ------------------------------------------------------------------
// Recent tasks
// ------------------------------------------------------------------
export const RECENT_TASKS: RecentTask[] = [
  { id: 't-001', title: 'Refactor authentication module — JWT token rotation', status: 'completed', finished_iso: new Date(hours(3)).toISOString(), fsm_state_at_end: 'COMPLETE', duration_ms: 5_120_000 },
  { id: 't-002', title: 'Fix BM25 p99 latency regression in knowledge crate', status: 'completed', finished_iso: new Date(days(1)).toISOString(), fsm_state_at_end: 'COMPLETE', duration_ms: 2_340_000 },
  { id: 't-003', title: 'Add DoLa confidence calibration to sidecar server', status: 'interrupted', finished_iso: new Date(days(2)).toISOString(), fsm_state_at_end: 'EXECUTING', duration_ms: 8_900_000 },
  { id: 't-004', title: 'Scaffold WebContentIndex with ARAG tier 3 pipeline', status: 'completed', finished_iso: new Date(days(4)).toISOString(), fsm_state_at_end: 'COMPLETE', duration_ms: 3_600_000 },
  { id: 't-005', title: 'Implement BSP superstep snapshot serialization', status: 'failed', finished_iso: new Date(days(7)).toISOString(), fsm_state_at_end: 'AUDITING', duration_ms: 1_800_000 },
];

// ------------------------------------------------------------------
// FSM snapshot
// ------------------------------------------------------------------
export const FSM_SNAPSHOT: FsmSnapshot = {
  state: 'EXECUTING',
  current_step_id: 'bs-3.2.1.4',
  progress: 0.47,
  history: [
    { from: 'IDLE', to: 'PLANNING', ts: hours(4), trigger: 'submit_task' },
    { from: 'PLANNING', to: 'FEASIBILITY_PROBE', ts: hours(3.8), trigger: 'plan_complete' },
    { from: 'FEASIBILITY_PROBE', to: 'TESTING_FIRST', ts: hours(3.5), trigger: 'feasibility_pass' },
    { from: 'TESTING_FIRST', to: 'EXECUTING', ts: hours(3.2), trigger: 'tests_written' },
  ],
  agent_substate: 'executor consuming 42% GRC budget',
};

// ------------------------------------------------------------------
// Decomposition tree: "Refactor authentication module"
// Phase 3 active, phases 1-2 done
// ------------------------------------------------------------------
export const DECOMP_TREE: DecompNode = {
  id: 'c-1',
  level: 'concept',
  label: 'Refactor authentication module',
  status: 'active',
  elapsed_ms: 5_120_000,
  children: [
    {
      id: 'p-1',
      level: 'phase',
      label: 'Phase 1 — Audit existing auth paths',
      status: 'done',
      elapsed_ms: 1_200_000,
      children: [
        {
          id: 's-1.1', level: 'step', label: 'Map JWT token issuance call sites', status: 'done', elapsed_ms: 380_000,
          children: [
            { id: 'bs-1.1.1', level: 'baby-step', label: 'Grep all jwt::encode call sites', status: 'done', elapsed_ms: 42_000, detail: { blocks: [ { block: 'GROUNDING', status: 'done' }, { block: 'PRE_GATE', status: 'done' }, { block: 'TASK', status: 'done' }, { block: 'TDD', status: 'done' }, { block: 'DETERMINISTIC_CHECK', status: 'done' }, { block: 'ACCEPTANCE', status: 'done' }, { block: 'INTEGRATION', status: 'done' }, { block: 'EXIT_CRITERIA', status: 'done' } ] } },
            { id: 'bs-1.1.2', level: 'baby-step', label: 'Verify token expiry config read paths', status: 'done', elapsed_ms: 38_000 },
            { id: 'bs-1.1.3', level: 'baby-step', label: 'Document token scope matrix in DKG', status: 'done', elapsed_ms: 62_000 },
          ],
        },
        {
          id: 's-1.2', level: 'step', label: 'Identify refresh token storage strategy', status: 'done', elapsed_ms: 420_000,
          children: [
            { id: 'bs-1.2.1', level: 'baby-step', label: 'Read RefreshTokenStore trait implementations', status: 'done', elapsed_ms: 55_000 },
            { id: 'bs-1.2.2', level: 'baby-step', label: 'Audit Redis TTL alignment with token expiry', status: 'done', elapsed_ms: 48_000 },
            { id: 'bs-1.2.3', level: 'baby-step', label: 'Write contract test for token invalidation', status: 'done', elapsed_ms: 71_000 },
          ],
        },
        {
          id: 's-1.3', level: 'step', label: 'Baseline harness run — auth crate', status: 'done', elapsed_ms: 220_000,
          children: [
            { id: 'bs-1.3.1', level: 'baby-step', label: 'Run A-1 baseline against crates/auth', status: 'done', elapsed_ms: 180_000 },
            { id: 'bs-1.3.2', level: 'baby-step', label: 'Record 7 metrics in baseline_results.toml', status: 'done', elapsed_ms: 25_000 },
            { id: 'bs-1.3.3', level: 'baby-step', label: 'Check attention ceiling delta vs threshold', status: 'done', elapsed_ms: 15_000 },
          ],
        },
      ],
    },
    {
      id: 'p-2',
      level: 'phase',
      label: 'Phase 2 — Design rotation protocol',
      status: 'done',
      elapsed_ms: 1_800_000,
      children: [
        {
          id: 's-2.1', level: 'step', label: 'Define RotationPolicy trait', status: 'done', elapsed_ms: 640_000,
          children: [
            { id: 'bs-2.1.1', level: 'baby-step', label: 'Write GROUNDING for RFC 7519 section 4.1.4', status: 'done', elapsed_ms: 90_000 },
            { id: 'bs-2.1.2', level: 'baby-step', label: 'Implement RotationPolicy trait in auth crate', status: 'done', elapsed_ms: 280_000 },
            { id: 'bs-2.1.3', level: 'baby-step', label: 'Write deterministic check: no panics on rotation', status: 'done', elapsed_ms: 95_000 },
          ],
        },
        {
          id: 's-2.2', level: 'step', label: 'Integrate policy into token middleware', status: 'done', elapsed_ms: 720_000,
          children: [
            { id: 'bs-2.2.1', level: 'baby-step', label: 'Patch tower::Service middleware to call policy.rotate()', status: 'done', elapsed_ms: 310_000 },
            { id: 'bs-2.2.2', level: 'baby-step', label: 'Add integration checkpoint BABY_STEP level', status: 'done', elapsed_ms: 85_000 },
            { id: 'bs-2.2.3', level: 'baby-step', label: 'Acceptance gate: token lifetime bounded by policy', status: 'done', elapsed_ms: 120_000 },
          ],
        },
        {
          id: 's-2.3', level: 'step', label: 'Audit pass for Phase 2', status: 'done', elapsed_ms: 380_000,
          children: [
            { id: 'bs-2.3.1', level: 'baby-step', label: 'Run auditor with contract tests/auth/', status: 'done', elapsed_ms: 210_000 },
            { id: 'bs-2.3.2', level: 'baby-step', label: 'Memorize Phase 2 findings to DKG', status: 'done', elapsed_ms: 95_000 },
            { id: 'bs-2.3.3', level: 'baby-step', label: 'Transition FSM AUDITING -> EXECUTING', status: 'done', elapsed_ms: 18_000 },
          ],
        },
      ],
    },
    {
      id: 'p-3',
      level: 'phase',
      label: 'Phase 3 — Implement token rotation (ACTIVE)',
      status: 'active',
      elapsed_ms: 1_420_000,
      children: [
        {
          id: 's-3.1', level: 'step', label: 'Implement sliding-window refresh logic', status: 'done', elapsed_ms: 580_000,
          children: [
            { id: 'bs-3.1.1', level: 'baby-step', label: 'Write TDD spec for sliding window rotation', status: 'done', elapsed_ms: 95_000 },
            { id: 'bs-3.1.2', level: 'baby-step', label: 'Implement SlidingWindowPolicy in crates/auth/src/rotation.rs', status: 'done', elapsed_ms: 320_000 },
            { id: 'bs-3.1.3', level: 'baby-step', label: 'Deterministic check: rotation not lossy under load', status: 'done', elapsed_ms: 140_000 },
          ],
        },
        {
          id: 's-3.2', level: 'step', label: 'Add token revocation store', status: 'active', elapsed_ms: 420_000,
          children: [
            { id: 'bs-3.2.1', level: 'baby-step', label: 'Define RevocationStore trait with get/insert/purge', status: 'done', elapsed_ms: 85_000 },
            { id: 'bs-3.2.2', level: 'baby-step', label: 'Implement RedisRevocationStore with TTL sweep', status: 'done', elapsed_ms: 220_000 },
            { id: 'bs-3.2.3', level: 'baby-step', label: 'Write INTEGRATION checkpoint for revocation round-trip', status: 'done', elapsed_ms: 72_000 },
            {
              id: 'bs-3.2.4', level: 'baby-step', label: 'Add EXIT_CRITERIA: revoked token rejected in <2ms',
              status: 'active',
              elapsed_ms: 43_000,
              detail: {
                blocks: [
                  { block: 'GROUNDING', status: 'done' },
                  { block: 'PRE_GATE', status: 'done' },
                  { block: 'TASK', status: 'active' },
                  { block: 'TDD', status: 'pending' },
                  { block: 'DETERMINISTIC_CHECK', status: 'pending' },
                  { block: 'ACCEPTANCE', status: 'pending' },
                  { block: 'INTEGRATION', status: 'pending' },
                  { block: 'EXIT_CRITERIA', status: 'pending' },
                ],
              },
            },
          ],
        },
        {
          id: 's-3.3', level: 'step', label: 'Clock-skew tolerance and replay protection', status: 'pending', elapsed_ms: 0,
          children: [
            { id: 'bs-3.3.1', level: 'baby-step', label: 'Add clock_skew_tolerance_ms config field', status: 'pending', elapsed_ms: 0 },
            { id: 'bs-3.3.2', level: 'baby-step', label: 'Implement replay window with bloom filter', status: 'pending', elapsed_ms: 0 },
            { id: 'bs-3.3.3', level: 'baby-step', label: 'Property test: replay never succeeds within window', status: 'pending', elapsed_ms: 0 },
          ],
        },
      ],
    },
    {
      id: 'p-4',
      level: 'phase',
      label: 'Phase 4 — Integration + E2E',
      status: 'pending',
      elapsed_ms: 0,
      children: [
        { id: 's-4.1', level: 'step', label: 'E2E test harness for auth flow', status: 'pending', elapsed_ms: 0, children: [
          { id: 'bs-4.1.1', level: 'baby-step', label: 'Write Playwright test: login -> refresh -> revoke', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-4.1.2', level: 'baby-step', label: 'Assert VRAM stays below 10.8 GB during E2E', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-4.1.3', level: 'baby-step', label: 'Run full test suite with -p auth', status: 'pending', elapsed_ms: 0 },
        ]},
        { id: 's-4.2', level: 'step', label: 'Memorize and complete', status: 'pending', elapsed_ms: 0, children: [
          { id: 'bs-4.2.1', level: 'baby-step', label: 'Spawn memory-writer agent', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-4.2.2', level: 'baby-step', label: 'Transition MEMORIZING -> COMPLETE -> IDLE', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-4.2.3', level: 'baby-step', label: 'Release session file locks via mcp_session_release', status: 'pending', elapsed_ms: 0 },
        ]},
      ],
    },
    {
      id: 'p-5',
      level: 'phase',
      label: 'Phase 5 — Docs + Changelog',
      status: 'pending',
      elapsed_ms: 0,
      children: [
        { id: 's-5.1', level: 'step', label: 'Update operatingManual.md with rotation semantics', status: 'pending', elapsed_ms: 0, children: [
          { id: 'bs-5.1.1', level: 'baby-step', label: 'Write changelog.md entries for all changed files', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-5.1.2', level: 'baby-step', label: 'Update activeContext.md under 80 lines', status: 'pending', elapsed_ms: 0 },
          { id: 'bs-5.1.3', level: 'baby-step', label: 'Commit with patch-X.Y prefix and test results', status: 'pending', elapsed_ms: 0 },
        ]},
      ],
    },
  ],
};

// ------------------------------------------------------------------
// BubbleUp events
// ------------------------------------------------------------------
export const BUBBLE_UP_EVENTS: BubbleUpEvent[] = [
  { ts: mins(12), level: 'baby-step', node_id: 'bs-3.2.3', message: 'EXIT_CRITERIA met: revocation round-trip < 1.2ms', escalated: false },
  { ts: mins(8), level: 'step', node_id: 's-3.2', message: 'STEP checkpoint PARTIAL — bs-3.2.4 still active', escalated: false },
  { ts: mins(5), level: 'phase', node_id: 'p-3', message: 'Phase 3 progress: 2/3 steps complete', escalated: false },
  { ts: mins(3), level: 'concept', node_id: 'c-1', message: 'Concept health: NOMINAL — no escalation required', escalated: false },
];

// ------------------------------------------------------------------
// Activity entries — 25 entries across 3 step headers
// ------------------------------------------------------------------
export const ACTIVITY_ENTRIES: ActivityEntry[] = [
  // --- Step header: s-3.1 (done) ---
  { id: 'ae-001', kind: 'phase-transition', ts: hours(1.2), step_id: 's-3.1', payload: { kind: 'phase-transition', from_state: 'TESTING_FIRST', to_state: 'EXECUTING', trigger: 'tests_written' } },
  { id: 'ae-002', kind: 'reasoning', ts: hours(1.1), step_id: 's-3.1', agent: 'executor', payload: { kind: 'reasoning', step_type: 'observe', content: 'SlidingWindowPolicy requires a monotonic clock — std::time::Instant is not suitable across process restarts. Must use tokio::time::Instant or a seeded u64 epoch.', grounding_source: 'code we just read: crates/auth/src/rotation.rs:42', confidence: 'Verified', call_num: 1, budget_remaining: 1197 } },
  { id: 'ae-003', kind: 'reasoning', ts: hours(1.05), step_id: 's-3.1', agent: 'executor', payload: { kind: 'reasoning', step_type: 'hypothesize', content: 'Use unix_timestamp_millis from SystemTime::now() gated by a monotonic guard for same-millisecond ordering.', grounding_source: 'code we just read', confidence: 'High confidence', call_num: 2, budget_remaining: 1053 } },
  { id: 'ae-004', kind: 'edit', ts: hours(1.0), step_id: 's-3.1', agent: 'executor', payload: { kind: 'edit', file: 'crates/auth/src/rotation.rs', added: 47, removed: 12, diff_summary: 'Replace Instant usage with SystemTime epoch guard; add monotonic_ms() helper.', expanded: false } },
  { id: 'ae-005', kind: 'test', ts: mins(55), step_id: 's-3.1', agent: 'auditor', payload: { kind: 'test', pass: 18, fail: 0, crate: 'auth', failures: [] } },
  { id: 'ae-006', kind: 'validate', ts: mins(52), step_id: 's-3.1', agent: 'auditor', payload: { kind: 'validate', tier: 1, verdict: 'ALLOW', rule_id: 'R-112', message: 'No .unwrap() in production path — T1 PASS' } },
  { id: 'ae-007', kind: 'validate', ts: mins(51), step_id: 's-3.1', agent: 'auditor', payload: { kind: 'validate', tier: 2, verdict: 'ALLOW', rule_id: 'R-218', message: 'blake3 used for token hash — T2 PASS' } },
  { id: 'ae-008', kind: 'checkpoint', ts: mins(48), step_id: 's-3.1', payload: { kind: 'checkpoint', level: 'STEP', result: 'PASS', baby_step_id: 'bs-3.1.3' } },

  // --- Step header: s-3.2 (active) ---
  { id: 'ae-009', kind: 'reasoning', ts: mins(44), step_id: 's-3.2', agent: 'executor', payload: { kind: 'reasoning', step_type: 'plan_action', content: 'RevocationStore trait needs get(token_hash) -> Option<bool>, insert(hash, ttl_ms) -> Result<()>, purge_expired() -> Result<usize>. No generics beyond hash type.', grounding_source: 'code we just read: crates/auth/src/lib.rs:88', confidence: 'Verified', call_num: 1, budget_remaining: 1190 } },
  { id: 'ae-010', kind: 'edit', ts: mins(41), step_id: 's-3.2', agent: 'executor', payload: { kind: 'edit', file: 'crates/auth/src/revocation.rs', added: 82, removed: 0, diff_summary: 'New file: RevocationStore trait + NoopRevocationStore impl.', expanded: false } },
  { id: 'ae-011', kind: 'edit', ts: mins(38), step_id: 's-3.2', agent: 'executor', payload: { kind: 'edit', file: 'crates/auth/src/redis_revocation.rs', added: 134, removed: 0, diff_summary: 'RedisRevocationStore: uses deadpool-redis, TTL sweep via SCAN pattern.', expanded: false } },
  { id: 'ae-012', kind: 'reasoning', ts: mins(35), step_id: 's-3.2', agent: 'executor', payload: { kind: 'reasoning', step_type: 'act', content: 'Implementing purge_expired as a background tokio::spawn task with 60s interval to avoid blocking hot path.', grounding_source: 'code we just read', confidence: 'High confidence', call_num: 3, budget_remaining: 910 } },
  { id: 'ae-013', kind: 'test', ts: mins(30), step_id: 's-3.2', agent: 'auditor', payload: { kind: 'test', pass: 23, fail: 1, crate: 'auth', failures: ['test_redis_revocation_ttl_precision'] } },
  { id: 'ae-014', kind: 'reasoning', ts: mins(28), step_id: 's-3.2', agent: 'executor', payload: { kind: 'reasoning', step_type: 'verify', content: 'test_redis_revocation_ttl_precision fails because test uses tokio::time::sleep which is not real-time in test mode. Must use tokio::time::pause() + advance().', grounding_source: 'code we just read: tests/revocation_tests.rs:55', confidence: 'Verified', call_num: 4, budget_remaining: 780 } },
  { id: 'ae-015', kind: 'edit', ts: mins(25), step_id: 's-3.2', agent: 'executor', payload: { kind: 'edit', file: 'tests/revocation_tests.rs', added: 8, removed: 3, diff_summary: 'Replace sleep with time::pause()+advance() for deterministic TTL test.', expanded: false } },
  { id: 'ae-016', kind: 'test', ts: mins(22), step_id: 's-3.2', agent: 'auditor', payload: { kind: 'test', pass: 24, fail: 0, crate: 'auth', failures: [] } },
  { id: 'ae-017', kind: 'validate', ts: mins(20), step_id: 's-3.2', agent: 'auditor', payload: { kind: 'validate', tier: 3, verdict: 'WARN', rule_id: 'R-341', message: 'deadpool-redis version 0.11 — nightly dep audit pending. Not blocked.' } },
  { id: 'ae-018', kind: 'cloud', ts: mins(18), step_id: 's-3.2', agent: 'researcher', payload: { kind: 'cloud', tier: 'claude-3-5-sonnet-20241022', reason: 'Context exceeded local 25K window during redis pool sizing research', tokens_used: 4820 } },
  { id: 'ae-019', kind: 'checkpoint', ts: mins(15), step_id: 's-3.2', payload: { kind: 'checkpoint', level: 'BABY_STEP', result: 'PASS', baby_step_id: 'bs-3.2.3' } },
  { id: 'ae-020', kind: 'mistake', ts: mins(13), step_id: 's-3.2', agent: 'executor', payload: { kind: 'mistake', hash: '3a7f9c2e1b8d4f6a0e5c9d2b7f3a1e8c', pattern_id: 'PATTERN-042', description: 'Used tokio::sync::Mutex in synchronous context — replaced with std::sync::Mutex after audit flag.', severity: 'WARNING' } },
  { id: 'ae-021', kind: 'preview', ts: mins(10), step_id: 's-3.2', agent: 'executor', payload: { kind: 'preview', url: 'http://localhost:8080/health', label: 'auth sidecar health check' } },

  // --- Step header: bs-3.2.4 (active) ---
  { id: 'ae-022', kind: 'reasoning', ts: mins(8), step_id: 'bs-3.2.4', agent: 'executor', payload: { kind: 'reasoning', step_type: 'observe', content: 'EXIT_CRITERIA requires revoked token rejected in <2ms. Current p50 is 1.1ms, p99 is 3.4ms — p99 exceeds target. Must investigate.', grounding_source: 'metrics/phase_3.toml: latency_p99=3.4ms', confidence: 'Verified', call_num: 1, budget_remaining: 1199 } },
  { id: 'ae-023', kind: 'reasoning', ts: mins(6), step_id: 'bs-3.2.4', agent: 'executor', payload: { kind: 'reasoning', step_type: 'hypothesize', content: 'p99 spike caused by Redis connection pool exhaustion under burst load. deadpool max_size=5 may be too low for concurrent auth checks.', grounding_source: 'code we just read: crates/auth/src/redis_revocation.rs:28', confidence: 'High confidence', call_num: 2, budget_remaining: 1048 } },
  { id: 'ae-024', kind: 'edit', ts: mins(4), step_id: 'bs-3.2.4', agent: 'executor', payload: { kind: 'edit', file: 'crates/auth/src/redis_revocation.rs', added: 6, removed: 2, diff_summary: 'Increase deadpool max_size from 5 to 20; add connection timeout 100ms.', expanded: false } },
  { id: 'ae-025', kind: 'reasoning', ts: mins(2), step_id: 'bs-3.2.4', agent: 'executor', payload: { kind: 'reasoning', step_type: 'act', content: 'Re-running latency benchmark after pool size increase. Expecting p99 to drop below 2ms threshold.', grounding_source: 'code we just read', confidence: 'High confidence', call_num: 3, budget_remaining: 900 } },
];

// ------------------------------------------------------------------
// Command cards — 6 cards spanning all types
// ------------------------------------------------------------------
export const COMMAND_CARDS: CommandCard[] = [
  {
    id: 'cc-001', ts: hours(3.5), kind: 'user',
    payload: { kind: 'user', text: 'Refactor the authentication module. Focus on JWT token rotation with sliding-window policy. All existing tests must stay green.', intent: 'TASK' },
  },
  {
    id: 'cc-002', ts: hours(3.4), kind: 'status',
    payload: { kind: 'status', message: 'Task accepted. FSM transitioning IDLE -> PLANNING. Context compiler spawned.', fsm_state: 'PLANNING', step_id: undefined },
  },
  {
    id: 'cc-003', ts: hours(2.1), kind: 'disambiguation',
    payload: { kind: 'disambiguation', question: 'RevocationStore backend: which should I target?', options: [ { id: 'opt-a', label: 'Redis (deadpool-redis)' }, { id: 'opt-b', label: 'In-memory (DashMap, single-process only)' }, { id: 'opt-c', label: 'Both, behind a trait (recommended)' } ] },
  },
  {
    id: 'cc-004', ts: mins(30), kind: 'error',
    payload: { kind: 'error', message: 'test_redis_revocation_ttl_precision FAILED: assertion left==right. left=false, right=true at tests/revocation_tests.rs:62', source: 'crates/auth', step_id: 'bs-3.2.2', recoverable: true },
  },
  {
    id: 'cc-005', ts: mins(18), kind: 'steering',
    payload: { kind: 'steering', vector_label: 'latency-sensitivity-v3', delta_description: 'Steering vector shifted: elevated weight on latency-critical paths. p99 monitoring threshold tightened from 5ms to 2ms.', applied: true },
  },
  {
    id: 'cc-006', ts: mins(8), kind: 'intervention',
    payload: {
      kind: 'intervention',
      request: {
        id: 'int-001', title: 'Approve deadpool-redis pool size change', description: 'Executor proposes increasing max_size from 5 to 20 connections. This affects resource consumption on low-VRAM machines (8GB tier).', file: 'crates/auth/src/redis_revocation.rs', diff_summary: '- max_size: 5\n+ max_size: 20\n- connection_timeout: None\n+ connection_timeout: Some(Duration::from_millis(100))', created_ts: mins(9),
      },
    },
  },
];

// ------------------------------------------------------------------
// Active intervention request
// ------------------------------------------------------------------
export const ACTIVE_INTERVENTION: InterventionRequest = {
  id: 'int-001',
  title: 'Approve deadpool-redis pool size change',
  description: 'Executor proposes increasing max_size from 5 to 20 connections. This affects resource consumption on low-VRAM machines (8GB tier). p99 latency is currently 3.4ms, target <2ms.',
  file: 'crates/auth/src/redis_revocation.rs',
  diff_summary: '- max_size: 5\n+ max_size: 20\n- connection_timeout: None\n+ connection_timeout: Some(Duration::from_millis(100))',
  created_ts: mins(9),
};

// ------------------------------------------------------------------
// Evidence gates — 4 pass, 1 block, 1 warn, 1 pending
// ------------------------------------------------------------------
export const EVIDENCE_GATES: EvidenceGate[] = [
  { id: 'eg-001', name: 'Grounding sweep complete', verdict: 'pass', reason: '28 grounded, 12 verify-needed, 5 ungrounded — within threshold', checked_at: mins(20) },
  { id: 'eg-002', name: 'Feasibility probe passed', verdict: 'pass', reason: 'All 4 feasibility checks PASS: deps available, VRAM sufficient, schema valid, contract tests written', checked_at: hours(3.5) },
  { id: 'eg-003', name: 'TDD tests written before implementation', verdict: 'pass', reason: 'Contract tests written for RevocationStore trait before any impl code', checked_at: hours(2.8) },
  { id: 'eg-004', name: 'No .unwrap() in production path', verdict: 'pass', reason: 'T1 audit ALLOW across all modified files', checked_at: mins(22) },
  { id: 'eg-005', name: 'dep audit — deadpool-redis', verdict: 'block', reason: 'deadpool-redis 0.11 has a CVE-2025-1234 advisory (connection leak under TLS). Must upgrade to 0.12+.', checked_at: mins(17) },
  { id: 'eg-006', name: 'p99 latency target < 2ms', verdict: 'warn', reason: 'Current p99=3.4ms, target=2ms. Pool size fix in progress (bs-3.2.4).', checked_at: mins(5) },
  { id: 'eg-007', name: 'Memory-writer memorize complete', verdict: 'pending', reason: 'Phase 3 not yet complete — memorize not triggered', checked_at: undefined },
];

// ------------------------------------------------------------------
// GRC budgets — 6 roles
// ------------------------------------------------------------------
export const GRC_BUDGETS: GrcBudget[] = [
  { role: 'executor', total_calls: 8, total_chars: 1200, calls_used: 3, chars_used: 420, calls_remaining: 5, chars_remaining: 780 },
  { role: 'auditor', total_calls: 10, total_chars: 1200, calls_used: 6, chars_used: 650, calls_remaining: 4, chars_remaining: 550 },
  { role: 'context-compiler', total_calls: 5, total_chars: 1000, calls_used: 5, chars_used: 980, calls_remaining: 0, chars_remaining: 20 },
  { role: 'memory-writer', total_calls: 3, total_chars: 600, calls_used: 0, chars_used: 0, calls_remaining: 3, chars_remaining: 600 },
  { role: 'prompt-engineer', total_calls: 5, total_chars: 1000, calls_used: 2, chars_used: 310, calls_remaining: 3, chars_remaining: 690 },
  { role: 'researcher', total_calls: 8, total_chars: 1500, calls_used: 4, chars_used: 720, calls_remaining: 4, chars_remaining: 780 },
];

// ------------------------------------------------------------------
// Grounding sweep — 28 grounded, 12 verify-needed, 5 ungrounded
// ------------------------------------------------------------------
export const GROUNDING_SWEEP: GroundingSweep = {
  grounded: 28,
  verify_needed: 12,
  ungrounded: 5,
  claims: [
    { id: 'gc-001', text: 'deadpool-redis max_size=20 is safe on 12GB VRAM systems', classification: 'VERIFY_NEEDED', source: 'training (unsearched)' },
    { id: 'gc-002', text: 'tokio::time::pause() works in multi-threaded runtime', classification: 'GROUNDED', source: 'code we just read: tokio docs' },
    { id: 'gc-003', text: 'blake3 is faster than sha2 for short token hashes', classification: 'GROUNDED', source: 'WebSearch: benchmarks.rs 2025' },
    { id: 'gc-004', text: 'Redis SCAN is O(1) amortized over keyspace', classification: 'VERIFY_NEEDED', source: 'training (unsearched)' },
    { id: 'gc-005', text: 'SystemTime::now() is monotonic on Windows', classification: 'UNGROUNDED', source: undefined },
  ],
};

// ------------------------------------------------------------------
// CSQ entries — 2 Critical pending guard
// ------------------------------------------------------------------
export const CSQ_PENDING: CsqEntry[] = [
  { id: 'csq-001', severity: 'CRITICAL', question: 'If purge_expired() blocks the async runtime, revocation check latency will spike under concurrent load.', guard_added: false, function_context: 'crates/auth/src/redis_revocation.rs:purge_expired' },
  { id: 'csq-002', severity: 'CRITICAL', question: 'If connection pool exhausts during burst, get() returns Err — callers must handle this as a reject (fail-closed), not a bypass.', guard_added: false, function_context: 'crates/auth/src/redis_revocation.rs:get' },
];

// ------------------------------------------------------------------
// Embedding circuit breakers — 4 closed, 1 half-open
// ------------------------------------------------------------------
export const EMBEDDING_BREAKERS: EmbeddingBreaker[] = [
  { model: 'graphcodebert-base', state: 'CLOSED', failures: 0, success_rate: 1.0 },
  { model: 'granite-embedding-30m', state: 'CLOSED', failures: 1, success_rate: 0.998 },
  { model: 'cross-encoder-ms-marco-MiniLM', state: 'HALF_OPEN', failures: 4, success_rate: 0.91, last_failure_ts: mins(14) },
  { model: 'splade-v3-lexical-expansion', state: 'CLOSED', failures: 0, success_rate: 1.0 },
  { model: 'hyde-hypothesis-generator', state: 'CLOSED', failures: 2, success_rate: 0.995 },
];

// ------------------------------------------------------------------
// Recall metrics — real ELAI numbers from DOMAIN-CONTEXT.md section 5
// DEEP MRR < STANDARD MRR per real metrics
// ------------------------------------------------------------------
export const RECALL_METRICS: RecallMetrics[] = [
  { tier: 'FAST', r_at_5: 0.45, r_at_10: 0.58, mrr: 0.32, latency_ms: 1 },
  { tier: 'STANDARD', r_at_5: 0.633, r_at_10: 0.767, mrr: 0.412, latency_ms: 8 },
  { tier: 'DEEP', r_at_5: 0.79, r_at_10: 0.86, mrr: 0.21, latency_ms: 84 },
];

// ------------------------------------------------------------------
// DKG stats — real numbers
// ------------------------------------------------------------------
export const DKG_STATS: DkgStats = {
  nodes: 8075,
  edges: 4148,
  bm25_chunks: 22_340,
  vector_status: 'READY',
  freshness_decay_pct: 12,
};

// ------------------------------------------------------------------
// Rule lifecycle counts
// ------------------------------------------------------------------
export const RULE_LIFECYCLE_COUNTS: RuleLifecycleCounts = {
  PROPOSED: 12,
  CANDIDATE: 8,
  SHADOW: 6,
  ACTIVE: 39,
  RETIRED: 21,
};

// ------------------------------------------------------------------
// Active rules (sample set for current EXECUTING state)
// ------------------------------------------------------------------
export const ACTIVE_RULES: Rule[] = [
  { id: 'R-101', name: 'No unwrap in production', lifecycle: 'ACTIVE', predicate: 'AST_PATTERN', domain: 'rust', active_in_states: ['EXECUTING', 'AUDITING'], added_ts: days(30) },
  { id: 'R-102', name: 'blake3 for all hashing', lifecycle: 'ACTIVE', predicate: 'GREP_PATTERN', domain: 'rust', active_in_states: ['EXECUTING', 'AUDITING', 'TESTING_FIRST'], added_ts: days(60) },
  { id: 'R-103', name: 'tokio::sync::Mutex in async', lifecycle: 'ACTIVE', predicate: 'AST_PATTERN', domain: 'rust', active_in_states: ['EXECUTING'], added_ts: days(45) },
  { id: 'R-112', name: 'No magic numbers', lifecycle: 'ACTIVE', predicate: 'AST_PATTERN', domain: 'rust', active_in_states: ['EXECUTING', 'AUDITING'], added_ts: days(20) },
  { id: 'R-218', name: 'deadpool over r2d2', lifecycle: 'ACTIVE', predicate: 'SCHEMA_REQUIRED', domain: 'rust', active_in_states: ['EXECUTING', 'TESTING_FIRST'], added_ts: days(15) },
  { id: 'R-341', name: 'dep audit on external crates', lifecycle: 'ACTIVE', predicate: 'VERSION_GATE', domain: 'rust', active_in_states: ['EXECUTING', 'AUDITING', 'COMPLETE'], added_ts: days(10) },
];

// ------------------------------------------------------------------
// Deprecated patterns
// ------------------------------------------------------------------
export const DEPRECATED_PATTERNS: DeprecatedPattern[] = [
  { id: 'dp-001', pattern: 'extern crate rand; use rand::Rng', language: 'Rust', added_ts: days(5), replacement: 'use rand::prelude::*; (rand >= 0.9)' },
  { id: 'dp-002', pattern: 'Box<dyn std::error::Error>', language: 'Rust', added_ts: days(12), replacement: 'thiserror::Error or anyhow::Error' },
  { id: 'dp-003', pattern: 'http.server.HTTPServer', language: 'Python', added_ts: days(8), replacement: 'FastAPI + uvicorn' },
  { id: 'dp-004', pattern: '#[async_trait]', language: 'Rust', added_ts: days(3), replacement: 'native async fn in traits (Rust 1.75+)' },
  { id: 'dp-005', pattern: 'torch.no_grad()', language: 'Python', added_ts: days(1), replacement: 'torch.inference_mode()' },
];

// ------------------------------------------------------------------
// Source tier distribution
// ------------------------------------------------------------------
export const SOURCE_TIER_DISTRIBUTION: SourceTierDistribution[] = [
  { tier: 'T1', label: 'Official SDK / passing tests', count: 18, pct: 38 },
  { tier: 'T2', label: 'Code read in this session', count: 14, pct: 30 },
  { tier: 'T3', label: 'Official docs / WebSearch', count: 9, pct: 19 },
  { tier: 'T4', label: 'Peer-reviewed papers', count: 4, pct: 8 },
  { tier: 'T5', label: 'Third-party benchmarks', count: 2, pct: 4 },
];

// ------------------------------------------------------------------
// VRAM — 1 GPU 12GB, 10240 used 2048 free
// ------------------------------------------------------------------
export const VRAM_GAUGES: VramGauge[] = [
  { gpu_id: 0, gpu_name: 'NVIDIA GeForce RTX 3060 12GB', total_mb: 12288, used_mb: 10240, free_mb: 2048, pct_used: 83 },
];

// ------------------------------------------------------------------
// Sidecar status
// ------------------------------------------------------------------
export const SIDECAR_STATUS: SidecarStatus = {
  loaded: true,
  model: 'Qwen2.5-Coder-7B-Instruct-abliterated',
  fa2: true,
  dola: false,
  steering: true,
  turboquant: 'TQ4',
  tokens_per_sec: 38.4,
  latency_p50_ms: 12,
  latency_p99_ms: 28,
  context_window: 25_000,
  mcp_bridge_active: false,
};

// ------------------------------------------------------------------
// Inference metrics
// ------------------------------------------------------------------
export const INFERENCE_METRICS: InferenceMetrics = {
  tokens_per_sec_last_60s: 38.4,
  latency_p50_ms: 12,
  latency_p99_ms: 28,
  total_tokens_session: 148_320,
  cloud_fallback_count: 1,
};

// ------------------------------------------------------------------
// GG sidecar signals
// ------------------------------------------------------------------
export const GG_SIGNALS: GgSignals = {
  mean_logprob: -0.184,
  min_logprob: -1.92,
  entropy_mean: 0.43,
  entropy_max: 1.88,
  token_count: 312,
};

// ------------------------------------------------------------------
// Context budget — 8 slots
// ------------------------------------------------------------------
export const CONTEXT_BUDGET: ContextBudget = {
  slots: [
    { name: 'system-prompt', used_tokens: 2_840, total_tokens: 4_000, pct: 71 },
    { name: 'memory-bank', used_tokens: 3_100, total_tokens: 5_000, pct: 62 },
    { name: 'plan-context', used_tokens: 1_820, total_tokens: 3_000, pct: 61 },
    { name: 'active-file', used_tokens: 4_200, total_tokens: 6_000, pct: 70 },
    { name: 'activity-stream', used_tokens: 2_400, total_tokens: 4_000, pct: 60 },
    { name: 'rag-results', used_tokens: 1_600, total_tokens: 3_000, pct: 53 },
    { name: 'tool-output', used_tokens: 880, total_tokens: 2_000, pct: 44 },
    { name: 'reserve', used_tokens: 200, total_tokens: 2_000, pct: 10 },
  ],
  total_used: 17_040,
  total_cap: 25_000,
};

// ------------------------------------------------------------------
// Agents
// ------------------------------------------------------------------
export const ACTIVE_AGENTS: AgentEntry[] = [
  { role: 'executor', lifecycle: 'RUNNING', pid: 44120, last_heartbeat_ms: 2_100, grc_budget_used_pct: 38 },
  { role: 'auditor', lifecycle: 'IDLE', pid: 44121, last_heartbeat_ms: 8_400, grc_budget_used_pct: 60 },
  { role: 'context-compiler', lifecycle: 'IDLE', pid: 44122, last_heartbeat_ms: 42_000, grc_budget_used_pct: 98 },
  { role: 'memory-writer', lifecycle: 'IDLE', pid: undefined, last_heartbeat_ms: 120_000, grc_budget_used_pct: 0 },
  { role: 'prompt-engineer', lifecycle: 'IDLE', pid: 44123, last_heartbeat_ms: 60_000, grc_budget_used_pct: 40 },
  { role: 'researcher', lifecycle: 'IDLE', pid: 44124, last_heartbeat_ms: 180_000, grc_budget_used_pct: 50 },
];

// ------------------------------------------------------------------
// Supervision log
// ------------------------------------------------------------------
export const SUPERVISION_LOG: SupervisionEvent[] = [
  { ts: hours(3.8), agent: 'context-compiler', event: 'STARTED', restart_count: 0 },
  { ts: hours(3.5), agent: 'executor', event: 'STARTED', restart_count: 0 },
  { ts: hours(3.2), agent: 'auditor', event: 'STARTED', restart_count: 0 },
  { ts: hours(2.1), agent: 'researcher', event: 'STARTED', restart_count: 0 },
  { ts: mins(45), agent: 'context-compiler', event: 'DIED', restart_count: 1, reason: 'OOM: context exceeded 25K token limit' },
  { ts: mins(44), agent: 'context-compiler', event: 'RESTARTED', restart_count: 1 },
  { ts: mins(40), agent: 'context-compiler', event: 'HEALTHY', restart_count: 1 },
];

// ------------------------------------------------------------------
// Heartbeats
// ------------------------------------------------------------------
export const HEARTBEATS: HeartbeatEntry[] = [
  { agent: 'executor', last_ts: mins(0.035), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
  { agent: 'auditor', last_ts: mins(0.14), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
  { agent: 'context-compiler', last_ts: mins(0.7), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
  { agent: 'memory-writer', last_ts: mins(2), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
  { agent: 'prompt-engineer', last_ts: mins(1), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
  { agent: 'researcher', last_ts: mins(3), interval_ms: 5_000, timeout_ms: 15_000, alive: true },
];

// ------------------------------------------------------------------
// Research tree
// ------------------------------------------------------------------
export const RESEARCH_TREE: ResearchTreeNode = {
  id: 'rt-1',
  query: 'Redis token revocation patterns and TTL strategies 2025',
  stage: 'CONVERGE',
  converged: true,
  phi: 0.82,
  psi: 0.71,
  source_count: 8,
  children: [
    { id: 'rt-1.1', query: 'deadpool-redis connection pooling best practices', stage: 'EXTRACT', converged: true, phi: 0.88, psi: 0.79, source_count: 3 },
    { id: 'rt-1.2', query: 'JWT revocation at scale — bloom filter vs Redis SCAN', stage: 'REFLECT', converged: false, phi: 0.61, psi: 0.45, source_count: 4 },
    { id: 'rt-1.3', query: 'clock skew tolerance in distributed auth systems', stage: 'SEARCH', converged: false, phi: 0.38, psi: 0.21, source_count: 1 },
  ],
};

// ------------------------------------------------------------------
// BSP supersteps
// ------------------------------------------------------------------
export const BSP_SUPERSTEPS: BspSuperstep[] = [
  { id: 1, ts: hours(1.05), baby_step_id: 'bs-3.1.1', block: 'TDD', duration_ms: 95_000, status: 'PASS' },
  { id: 2, ts: hours(0.9), baby_step_id: 'bs-3.1.2', block: 'TASK', duration_ms: 320_000, status: 'PASS' },
  { id: 3, ts: mins(52), baby_step_id: 'bs-3.1.3', block: 'DETERMINISTIC_CHECK', duration_ms: 140_000, status: 'PASS' },
  { id: 4, ts: mins(44), baby_step_id: 'bs-3.2.1', block: 'TASK', duration_ms: 85_000, status: 'PASS' },
  { id: 5, ts: mins(35), baby_step_id: 'bs-3.2.2', block: 'TASK', duration_ms: 220_000, status: 'PASS' },
  { id: 6, ts: mins(22), baby_step_id: 'bs-3.2.2', block: 'TDD', duration_ms: 184_000, status: 'PASS', notes: 'TTL test fixed: tokio::time::pause() substituted' },
  { id: 7, ts: mins(15), baby_step_id: 'bs-3.2.3', block: 'INTEGRATION', duration_ms: 72_000, status: 'PASS' },
  { id: 8, ts: mins(4), baby_step_id: 'bs-3.2.4', block: 'TASK', duration_ms: 0, status: 'SKIP', notes: 'In progress — pool resize applied, awaiting re-benchmark' },
];

// ------------------------------------------------------------------
// Session branches
// ------------------------------------------------------------------
export const SESSION_BRANCHES: SessionBranch[] = [
  { id: 'sb-001', label: 'main — auth-refactor', status: 'active', created_ts: hours(4), last_activity_ts: mins(2) },
  { id: 'sb-002', label: 'exploratory — bloom-filter-revocation', status: 'abandoned', created_ts: hours(2), last_activity_ts: hours(1.5), parent_branch_id: 'sb-001' },
  { id: 'sb-003', label: 'merged — baseline-harness-run', status: 'merged', created_ts: hours(3.8), last_activity_ts: hours(3.2), parent_branch_id: 'sb-001' },
];

// ------------------------------------------------------------------
// Anti-spiral counter
// ------------------------------------------------------------------
export const ANTI_SPIRAL_COUNTER: AntiSpiralCounter = {
  step_id: 'bs-3.2.4',
  edit_count: 2,
  threshold: 3,
  triggered: false,
  last_hash: 'f4a1c9e2b7d3f8a0c5e1b9d4f2a7c3e8',
};

// ------------------------------------------------------------------
// Mistake book
// ------------------------------------------------------------------
export const MISTAKE_BOOK: MistakeEntry[] = [
  { id: 'mb-001', baby_step_id: 'bs-3.2.2', hash: '3a7f9c2e1b8d4f6a0e5c9d2b7f3a1e8c', pattern_id: 'PATTERN-042', description: 'tokio::sync::Mutex used in synchronous context — replaced with std::sync::Mutex.', severity: 'WARNING', ts: mins(13), resolved: true },
  { id: 'mb-002', baby_step_id: 'bs-3.1.2', hash: 'b2e8f4a0c9d1e5b3f7a2c4d8e0b6f9a1', pattern_id: 'PATTERN-017', description: 'Magic number 5 used for pool size — extracted to named constant MAX_POOL_CONNECTIONS.', severity: 'WARNING', ts: hours(0.85), resolved: true },
  { id: 'mb-003', baby_step_id: 'bs-3.2.4', hash: '7d4b1e9f3a0c8d2e5b7f1a4c9d3e0b8f', pattern_id: 'PATTERN-091', description: 'SystemTime::now() assumed monotonic — not guaranteed on Windows. Logged for follow-up.', severity: 'CRITICAL', ts: mins(3), resolved: false },
];

// ------------------------------------------------------------------
// Executable pipeline DAG
// ------------------------------------------------------------------
export const PIPELINE_NODES: PipelineNode[] = [
  { id: 'pn-001', label: 'RevocationStore trait', kind: 'task', status: 'done', deps: [] },
  { id: 'pn-002', label: 'NoopRevocationStore', kind: 'task', status: 'done', deps: ['pn-001'] },
  { id: 'pn-003', label: 'RedisRevocationStore', kind: 'task', status: 'done', deps: ['pn-001'] },
  { id: 'pn-004', label: 'T1 audit gate', kind: 'gate', status: 'done', deps: ['pn-002', 'pn-003'] },
  { id: 'pn-005', label: 'Integration checkpoint', kind: 'gate', status: 'done', deps: ['pn-004'] },
  { id: 'pn-006', label: 'EXIT_CRITERIA: p99 <2ms', kind: 'gate', status: 'active', deps: ['pn-005'] },
  { id: 'pn-007', label: 'Merge to auth middleware', kind: 'merge', status: 'pending', deps: ['pn-006'] },
];

export const DAG_REPAIR_EVENTS: DagRepairEvent[] = [
  { ts: mins(28), kind: 'Substitute', from_node_id: 'pn-003', to_node_id: 'pn-003', t_dag_counter: 1, },
];

// ------------------------------------------------------------------
// Build slots + dev servers
// ------------------------------------------------------------------
export const BUILD_SLOTS: BuildSlot[] = [
  { id: 'bs-slot-001', label: 'auth-crate — draft', status: 'building', started_ts: mins(3) },
  { id: 'bs-slot-002', label: 'elai-sidecar — live', status: 'live', url: 'http://localhost:8080', duration_ms: 9_670, started_ts: hours(2) },
];

export const DEV_SERVERS: DevServer[] = [
  { pid: 44200, port: 8080, label: 'elai-sidecar', status: 'running', started_ts: hours(2), log_tail: ['[2026-04-25T10:04:02Z] INFO sidecar: loaded Qwen2.5-Coder-7B on cuda:0', '[2026-04-25T10:04:03Z] INFO sidecar: FA2 enabled, TQ4 active', '[2026-04-25T12:02:11Z] INFO sidecar: request 4820 tokens → 38.4 tok/s'] },
  { pid: 44201, port: 1420, label: 'elai-dashboard (dev)', status: 'running', started_ts: hours(3), log_tail: ['[vite] dev server ready at http://localhost:1420'] },
];

// ------------------------------------------------------------------
// Project intelligence
// ------------------------------------------------------------------
export const PROJECT_PROFILE: ProjectProfile = {
  tier1: { language: 'Rust', build_tool: 'Cargo', framework: 'Tokio + Axum', test_runner: 'cargo test + nextest', ci: 'GitHub Actions', linter: 'clippy + rustfmt' },
  tier2: { conventions: [
    { name: 'Error handling', strength: 'strong', example: 'thiserror::Error on all domain errors; anyhow in CLI layer' },
    { name: 'Async runtime', strength: 'strong', example: 'tokio everywhere; std::sync::Mutex only in provably sync-only contexts' },
    { name: 'Hashing', strength: 'strong', example: 'blake3 for all non-RFC hashing; sha2 only where RFC mandated' },
    { name: 'Test isolation', strength: 'moderate', example: 'tokio::time::pause() for time-sensitive tests; some tests still use sleep()' },
    { name: 'Naming conventions', strength: 'mixed', example: 'snake_case Rust, kebab-case CLI flags, but mixed in legacy auth/ module' },
  ] },
  tier3: {
    architecture_inference: '9-crate workspace: core → providers → knowledge → reasoning → rules → tools → plugins → orchestrator → cli. knowledge crate owns DKG + BM25 + embedding pipeline. orchestrator owns FSM + BSP. ipc-types is the shared message schema.',
    telescope_summaries: [
      { level: 1, summary: 'Rust multi-crate workspace implementing a deterministic dev engine with local LLM inference.' },
      { level: 2, summary: 'FSM-driven orchestration (9 states), knowledge retrieval (DKG+BM25+hybrid search), GRC reasoning chains, VRAM-tiered model selection.' },
      { level: 3, summary: 'Auth module: JWT issuance in crates/auth/src/token.rs, revocation in crates/auth/src/revocation.rs (in progress), middleware integration in crates/auth/src/middleware.rs.' },
      { level: 4, summary: 'Current baby step bs-3.2.4: RedisRevocationStore pool size tuning to achieve p99 < 2ms. pool max_size increased 5→20 with 100ms connection timeout. Awaiting benchmark re-run.' },
    ],
  },
};

// ------------------------------------------------------------------
// Feasibility report
// ------------------------------------------------------------------
export const FEASIBILITY_REPORT: FeasibilityReport = {
  overall: 'PASS',
  results: [
    { id: 'fr-001', check: 'Dependencies available in registry', outcome: 'PASS', detail: 'deadpool-redis 0.11 present (0.12 advisory pending)', ts: hours(3.5) },
    { id: 'fr-002', check: 'VRAM sufficient for inference + workspace', outcome: 'PASS', detail: '2048 MB free on cuda:0, model requires 1920 MB', ts: hours(3.5) },
    { id: 'fr-003', check: 'Contract tests written before implementation', outcome: 'PASS', detail: 'tests/contracts/revocation.rs: 4 tests present', ts: hours(3.4) },
    { id: 'fr-004', check: 'Schema valid: RevocationStore trait interface stable', outcome: 'PASS', detail: 'ipc-types/src/revocation.rs: no breaking changes', ts: hours(3.3) },
  ],
  probed_at: hours(3.5),
};

// ------------------------------------------------------------------
// Baseline results — A-1 harness 7 metrics
// ------------------------------------------------------------------
export const BASELINE_RESULTS: BaselineResults = {
  metrics: [
    { name: 'attention_ceiling', value: 0.91, unit: 'ratio', threshold: 0.85, pass: true },
    { name: 'retry_rate', value: 0.042, unit: 'ratio', threshold: 0.10, pass: true },
    { name: 'few_shot_delta', value: 0.08, unit: 'ratio', threshold: 0.05, pass: false },
    { name: 'bm25_p50_ms', value: 1.0, unit: 'ms', threshold: 5.0, pass: true },
    { name: 'bm25_p99_ms', value: 2.0, unit: 'ms', threshold: 10.0, pass: true },
    { name: 'r_at_5_standard', value: 0.633, unit: 'ratio', threshold: 0.60, pass: true },
    { name: 'build_time_s', value: 9.67, unit: 's', threshold: 30.0, pass: true },
  ],
  run_ts: hours(3.5),
  deterministic: true,
};

// ------------------------------------------------------------------
// Dep audit
// ------------------------------------------------------------------
export const DEP_AUDIT: DepAuditResult[] = [
  { crate_or_pkg: 'deadpool-redis', current_version: '0.11.7', latest_version: '0.12.1', outdated: true, security_advisory: 'CVE-2025-1234: connection leak under TLS mutual auth', checked_ts: hours(8) },
  { crate_or_pkg: 'tokio', current_version: '1.41.0', latest_version: '1.43.0', outdated: true, checked_ts: hours(8) },
  { crate_or_pkg: 'axum', current_version: '0.7.9', latest_version: '0.7.9', outdated: false, checked_ts: hours(8) },
  { crate_or_pkg: 'blake3', current_version: '1.5.4', latest_version: '1.5.4', outdated: false, checked_ts: hours(8) },
  { crate_or_pkg: 'rand', current_version: '0.9.0', latest_version: '0.9.0', outdated: false, checked_ts: hours(8) },
];

// ------------------------------------------------------------------
// MSI status
// ------------------------------------------------------------------
export const MSI_STATUS: MsiStatus = {
  session_id: 's-44100',
  owned_files: ['crates/auth/src/revocation.rs', 'crates/auth/src/redis_revocation.rs', 'tests/revocation_tests.rs'],
  locked_files: ['crates/auth/src/lib.rs', 'crates/auth/Cargo.toml'],
  conflicts: [],
  active_sessions: [{ id: 's-44100', task: 'Refactor authentication module' }],
};

// ------------------------------------------------------------------
// Steering status
// ------------------------------------------------------------------
export const STEERING_STATUS: SteeringStatus = {
  active: true,
  vector_label: 'latency-sensitivity-v3',
  last_computed_ts: mins(18),
  magnitude: 0.74,
  applied_count: 3,
};

// ------------------------------------------------------------------
// MCP status
// ------------------------------------------------------------------
export const MCP_STATUS: McpStatus = {
  enabled: false,
  bridge_url: 'http://localhost:8090/mcp',
  ping_ms: undefined,
  last_ping_ts: undefined,
  connected: false,
  tools_available: [],
};

// ------------------------------------------------------------------
// Gate status
// ------------------------------------------------------------------
export const GATE_STATUS: GateStatus = {
  mode: 'INTERACTIVE',
  auto_approve_threshold: 0.92,
  pending_count: 1,
  auto_advanced_count: 14,
};

// ------------------------------------------------------------------
// Model routing per role
// ------------------------------------------------------------------
export const MODEL_ROUTING: ModelRouting[] = [
  { role: 'executor', target: 'Qwen2.5-Coder-7B-Instruct-abliterated', fallback: 'claude-3-5-sonnet-20241022' },
  { role: 'auditor', target: 'Qwen2.5-Coder-7B-Instruct-abliterated', fallback: 'claude-3-5-sonnet-20241022' },
  { role: 'researcher', target: 'Qwen2.5-Coder-7B-Instruct-abliterated', fallback: 'claude-3-opus-20240229' },
  { role: 'designer', target: 'Qwen2.5-Coder-3B-Instruct-abliterated', fallback: undefined },
  { role: 'prompt-engineer', target: 'Qwen2.5-Coder-7B-Instruct-abliterated', fallback: undefined },
  { role: 'context-compiler', target: 'Qwen2.5-Coder-3B-Instruct-abliterated', fallback: undefined },
  { role: 'memory-writer', target: 'Qwen2.5-Coder-3B-Instruct-abliterated', fallback: undefined },
  { role: 'media-validator', target: 'Qwen2.5-Coder-3B-Instruct-abliterated', fallback: undefined },
];

// ------------------------------------------------------------------
// Auto-advance stats
// ------------------------------------------------------------------
export const AUTO_ADVANCE_STATS: AutoAdvanceStats = {
  total_steps: 22,
  auto_advanced: 14,
  intervention_required: 8,
  auto_advance_rate: 0.636,
  last_24h_rate: 0.72,
};

// ------------------------------------------------------------------
// Optimization proposals
// ------------------------------------------------------------------
export const OPTIMIZATION_PROPOSALS: OptimizationProposal[] = [
  { id: 'op-001', title: 'Upgrade deadpool-redis to 0.12.1', description: 'Fixes CVE-2025-1234 (TLS connection leak). No API changes required.', impact: 'HIGH', status: 'pending', created_ts: hours(8) },
  { id: 'op-002', title: 'Enable DoLa confidence calibration', description: 'DoLa showed +3% R@5 in STANDARD tier benchmarks. Toggle in sidecar settings.', impact: 'MEDIUM', status: 'pending', created_ts: days(2) },
  { id: 'op-003', title: 'Compress DKG edges with delta encoding', description: 'Estimated 18% reduction in DKG storage. Requires knowledge crate change.', impact: 'LOW', status: 'rejected', created_ts: days(5) },
];

// ------------------------------------------------------------------
// Query traces — recent 5
// ------------------------------------------------------------------
export const QUERY_TRACES: QueryTrace[] = [
  {
    id: 'qt-001', ts: mins(8), query: 'Redis connection pool max_size tuning', mode: 'HYBRID',
    stages: [ { name: 'BM25', latency_ms: 1, pass: true }, { name: 'Vector', latency_ms: 12, pass: true }, { name: 'CrossEncoder rerank', latency_ms: 34, pass: true } ],
    total_ms: 48, hit_count: 7,
  },
  {
    id: 'qt-002', ts: mins(20), query: 'tokio::time::pause async test patterns', mode: 'HYBRID',
    stages: [ { name: 'BM25', latency_ms: 1, pass: true }, { name: 'Vector', latency_ms: 11, pass: true }, { name: 'CrossEncoder rerank', latency_ms: 28, pass: true } ],
    total_ms: 41, hit_count: 4,
  },
  {
    id: 'qt-003', ts: mins(35), query: 'JWT revocation at scale bloom filter', mode: 'HYBRID',
    stages: [ { name: 'BM25', latency_ms: 2, pass: true }, { name: 'Vector', latency_ms: 15, pass: true }, { name: 'CrossEncoder rerank', latency_ms: 42, pass: false } ],
    total_ms: 60, hit_count: 3,
  },
  {
    id: 'qt-004', ts: mins(44), query: 'RevocationStore trait design patterns Rust', mode: 'BM25_ONLY',
    stages: [ { name: 'BM25', latency_ms: 1, pass: true } ],
    total_ms: 1, hit_count: 12,
  },
  {
    id: 'qt-005', ts: hours(1.1), query: 'SystemTime monotonic Windows guarantee', mode: 'HYBRID',
    stages: [ { name: 'BM25', latency_ms: 1, pass: true }, { name: 'Vector', latency_ms: 14, pass: true }, { name: 'CrossEncoder rerank', latency_ms: 31, pass: true } ],
    total_ms: 46, hit_count: 2,
  },
];

// ------------------------------------------------------------------
// Scaffold templates
// ------------------------------------------------------------------
export const SCAFFOLD_TEMPLATES: ScaffoldTemplate[] = [
  { id: 'st-001', label: 'Vite + Svelte 5', framework: 'svelte', files_count: 12 },
  { id: 'st-002', label: 'Vite + React 19', framework: 'react', files_count: 11 },
  { id: 'st-003', label: 'Astro 5 static', framework: 'astro', files_count: 9 },
  { id: 'st-004', label: 'Vanilla TS', framework: 'vanilla', files_count: 6 },
];

// ------------------------------------------------------------------
// WebARAG state
// ------------------------------------------------------------------
export const WEB_ARAG_STATE: WebAragState = {
  index_size: 142_000,
  last_crawled_ts: hours(6),
  pages_indexed: 284,
  quality_score: 0.78,
};

// ------------------------------------------------------------------
// Hook executions
// ------------------------------------------------------------------
export const HOOK_EXECUTIONS: HookExecution[] = [
  { ts: mins(4), hook: 'PreToolUse', tool: 'Edit', verdict: 'ALLOW' },
  { ts: mins(4), hook: 'PostToolUse', tool: 'Edit', verdict: 'ALLOW' },
  { ts: mins(22), hook: 'PostToolUse', tool: 'Bash', verdict: 'ALLOW' },
  { ts: mins(30), hook: 'PostToolUse', tool: 'Bash', verdict: 'WARN', reason: 'ruff: 1 style issue in test file (ignored)' },
  { ts: hours(3.8), hook: 'PreToolUse', tool: 'Write', verdict: 'ALLOW' },
  { ts: hours(3.5), hook: 'PreCompact', tool: undefined, verdict: 'ALLOW', reason: 'SESSION_HANDOFF saved to workflow/sessions/s-44100/handoff.md' },
];

// ------------------------------------------------------------------
// ShortCoder stats
// ------------------------------------------------------------------
export const SHORTCODER_STATS: ShortCoderStats = {
  savings_pct: 34,
  exact_hits: 48,
  whitespace_hits: 12,
  ellipsis_hits: 3,
  total_edits: 186,
};

// ------------------------------------------------------------------
// System status (composite)
// ------------------------------------------------------------------
export const SYSTEM_STATUS: SystemStatus = {
  fsm: FSM_SNAPSHOT,
  sidecar: SIDECAR_STATUS,
  vram: VRAM_GAUGES,
  gate: GATE_STATUS,
  active_agents: ACTIVE_AGENTS,
  session_id: 's-44100',
  uptime_ms: hours(4),
};
