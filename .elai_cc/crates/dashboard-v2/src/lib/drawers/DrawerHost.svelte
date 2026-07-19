<script lang="ts">
  import type { DrawerKey } from '../api/types';
  import KnowledgeDrawer from '../drawers/KnowledgeDrawer.svelte';
  import ReasoningDrawer from '../drawers/ReasoningDrawer.svelte';
  import OrchestrationDrawer from '../drawers/OrchestrationDrawer.svelte';
  import RulesDrawer from '../drawers/RulesDrawer.svelte';
  import ResearchDrawer from '../drawers/ResearchDrawer.svelte';
  import ProvidersDrawer from '../drawers/ProvidersDrawer.svelte';
  import ProjectIntelligenceDrawer from '../drawers/ProjectIntelligenceDrawer.svelte';
  import AntiStalenessDrawer from '../drawers/AntiStalenessDrawer.svelte';
  import ToolsHooksDrawer from '../drawers/ToolsHooksDrawer.svelte';
  import WebQualityDrawer from '../drawers/WebQualityDrawer.svelte';
  // B-42 — Cosmos artifact tier drawer.
  import CosmosDrawer from '../drawers/CosmosDrawer.svelte';
  // C-23 — Sessions drawer (BranchTree + per-branch SessionDag actions).
  import SessionsDrawer from '../drawers/SessionsDrawer.svelte';
  // D-1 — Reasoning template archive (MutationArchivePanel).
  import MutationArchivePanel from '../drawers/MutationArchivePanel.svelte';
  // D-2 — PPM verifier scores.
  import PpmScoresPanel from '../drawers/PpmScoresPanel.svelte';
  // D-3 — multi-verifier consensus.
  import VerificationConsensusPanel from '../drawers/VerificationConsensusPanel.svelte';
  // D-4 — tripwire stack panel.
  import TripwirePanel from '../drawers/TripwirePanel.svelte';
  // E-30 — public-agent-incident drawer (argv allowlist denies, trust
  // dialog defers, MCP pin refusals, lineage collisions, untrusted memory
  // write blocks, postmortems).
  import IncidentsDrawer from '../drawers/IncidentsDrawer.svelte';
  // D-19 — PRMBench adversarial-gaming benchmark panel.
  import PrmBenchPanel from '../drawers/PrmBenchPanel.svelte';
  // D-23 — Sycophancy detection benchmark panel.
  import SycophancyBenchPanel from '../drawers/SycophancyBenchPanel.svelte';
  // D-5 — BAI Thompson-sampling arms panel.
  import BaiArmsPanel from '../drawers/BaiArmsPanel.svelte';
  // D-6 — Held-out task store + 9-gram contamination filter panel.
  import HeldOutPanel from '../drawers/HeldOutPanel.svelte';
  // D-7 — Cross-domain transfer canary panel.
  import CanaryPanel from '../drawers/CanaryPanel.svelte';
  // D-9 — L3 frozen-surface registry + audit-run panel.
  import LockedKeysPanel from '../drawers/LockedKeysPanel.svelte';
  // D-10 — mode-routing predicate distribution + recent rows.
  import ModeRoutingPanel from '../drawers/ModeRoutingPanel.svelte';
  // D-21 — mode-routing coverage benchmark panel (macro F1 + confusion).
  import ModeRoutingBenchPanel from '../drawers/ModeRoutingBenchPanel.svelte';
  // D-11 — constructive-checker registry + recent results.
  import CheckerRegistryPanel from '../drawers/CheckerRegistryPanel.svelte';
  // D-13 — Adversary (cross-family) panel: counter-evidence + consensus
  // verdict transitions emitted by run_adversarial_pass.
  import AdversaryPanel from '../drawers/AdversaryPanel.svelte';
  // D-12 — OperationalizationCheck baby-step predicate panel: recent
  // baby_step_dispatched rows grouped by parent_task_id + pass rate +
  // per-checker-kind histogram.
  import BabyStepPanel from '../drawers/BabyStepPanel.svelte';
  // D-16 — Conformal calibration (UniCR) panel: tau + corpus + per-claim
  // abstain/answer decisions emitted by
  // conformal_calibration::compute_tau_and_emit + decide_and_emit.
  import ConformalCalibrationPanel from '../drawers/ConformalCalibrationPanel.svelte';
  // D-22 — Conformal calibration validation panel: per-bucket empirical
  // refusal rate + UniCR distribution-shift band gate emitted by
  // ConformalValidationRunner::run_all.
  import ConformalValidationPanel from '../drawers/ConformalValidationPanel.svelte';
  // D-15 — Discovery-vs-search NoveltyClass panel: recent
  // novelty_classified rows + per-class distribution. Producer:
  // elai_orchestrator::novelty::emit_novelty_classified +
  // agent_pool::dispatch_novelty.
  import NoveltyPanel from '../drawers/NoveltyPanel.svelte';
  // D-18 — Typed ObservableEffect predicate panel: recent
  // observable_effect_verified rows + per-sandbox-tier histogram + per-
  // outcome histogram + confound-flag chips. Producer:
  // elai_orchestrator::observable_effect::verify_observable_effect.
  import ObservableEffectPanel from '../drawers/ObservableEffectPanel.svelte';
  // D-17 — Honest-refusal SLA gauge: current rate + healthy-band
  // visualization + last breach chip + sparkline of recent rates.
  // Producer: elai_orchestrator::refusal_sla::RefusalRateMonitor::emit_snapshot,
  // reachable via agent_pool::dispatch_refusal_sla_check.
  import RefusalRateGauge from '../drawers/RefusalRateGauge.svelte';
  // D-20 — ARC-AGI-2 OOD probe panel: recent arc_benchmark_completed
  // rows + verdict banner + per-run dominant mode/novelty class.
  // Producer: elai_orchestrator::benchmarks::arc_agi2_runner::
  // ARCAGI2Runner::run_all, reachable via the `elai bench arc` CLI.
  import ARCBenchPanel from '../drawers/ARCBenchPanel.svelte';
  // E-46 — Hierarchical decomposition tuning panel (dual ledger +
  // AGENTLESS shortcuts + goal-drift canary).
  import PlanningLedgerPanel from '../drawers/PlanningLedgerPanel.svelte';
  // E-46 D-7 — Decomposition entropy histogram (low_entropy_decomposition rows).
  // Wire-dark unless ELAI_PLANNING_DECOMPOSITION_ENABLE=1.
  import DecompositionEntropyHistogram from '../drawers/DecompositionEntropyHistogram.svelte';
  // E-5 — User modeling + long-term memory drawer.
  import MemoryFactsDrawer from '../drawers/MemoryFactsDrawer.svelte';
  // E-1 dashboard slice — chat-memory (MemoryNotes + CoreBlock).
  import MemoryNotesPanel from '../drawers/MemoryNotesPanel.svelte';
  // E-35 — XGrammar grammar-cache visibility (panel lives in components/
  // because it is also embeddable inside other surfaces; DrawerHost mounts
  // it as a top-level drawer here so it has a wired route).
  import GrammarCompilePanel from '../components/GrammarCompilePanel.svelte';
  // D-26 — full-page archive explorer (search + lineage graph + composite
  // tripwire/BAI summary). Lives under views/ per the D-26 plan; mounted
  // here as a drawer key so the existing routing table covers it.
  import ArchiveExplorerView from '../views/ArchiveExplorerView.svelte';
  // D-27 — discovery-queue drawer (pending DiscoveryCandidate templates
  // pulled off the get_mode_routing_refusal_combined composite read).
  import DiscoveryQueueDrawer from '../drawers/DiscoveryQueueDrawer.svelte';
  // D-27 — full-page mode-routing + refusal-SLA dashboard view
  // (composition of D-10/D-15/D-16/D-17 panels + summary row).
  import ModeRoutingDashboardView from '../views/ModeRoutingDashboardView.svelte';
  // E-29 — Failure modes drawer (agentic observability: tool-bypass,
  // multi-step reasoning failures, attribution-loss, sycophantic-confirm,
  // OTel span breakages, all keyed off AgenticObservabilityEvent SSB rows).
  import FailureModesDrawer from '../drawers/FailureModesDrawer.svelte';
  // E-29 — Cloud spend / query trace / transcript drawers (Tier-A
  // observability surfaces, mounted adjacent to failure-modes).
  import SpendDrawer from '../drawers/SpendDrawer.svelte';
  import QueryDrawer from '../drawers/QueryDrawer.svelte';
  import TranscriptDrawer from '../drawers/TranscriptDrawer.svelte';
  // C-41 — Application Skills System inspector. Mounted as the 6th
  // SettingsWindow inspector card; also surfaced here so the DrawerKey
  // union stays exhaustive over DrawerHost's switch.
  import SkillsPanel from '../components/SkillsPanel.svelte';
  // F-1 / A5 — VibeGateDrawer (DrawerKey vibegate).
  // Wire-dark unless ELAI_VIBE_GATE_ENABLE=1 on the orchestrator side.
  import VibeGateDrawer from '../drawers/VibeGateDrawer.svelte';
  // F-2a / A5 — SecurityRulesPanel (DrawerKey security-rules).
  // Wire-dark unless ELAI_SECURITY_RULE_BLOCKED_ENABLE=1 on the orchestrator side.
  import SecurityRulesPanel from '../drawers/SecurityRulesPanel.svelte';
  // F-9 / Slice D5 — DepsSecurityPanel (DrawerKey deps-security).
  // Default-ON (B1/W-2): WriteTimeSlopsquatBlocked rows emit with no env override.
  import DepsSecurityPanel from '../drawers/DepsSecurityPanel.svelte';
  // F-10 / Slice A5 — SecurityMonotonicityDrawer (DrawerKey security-monotonicity).
  // Default-ON (W-2): SecurityClassRegressionBlocked rows emit unless the
  // ELAI_SECURITY_MONOTONICITY_DISABLE kill-switch is set.
  import SecurityMonotonicityDrawer from '../drawers/SecurityMonotonicityDrawer.svelte';
  // F-5 / Slice A5 — StalenessQuarantineDrawer (DrawerKey staleness-quarantine).
  // No ENABLE gate (W-2): staleness_quarantine_broadcast rows emit at the
  // memory-decay-tick boundary when a fact crosses the staleness threshold.
  import StalenessQuarantineDrawer from '../drawers/StalenessQuarantineDrawer.svelte';
  // F-7 / Slice A5 — TemplateRankingDrawer (DrawerKey template-ranking).
  // No ENABLE gate (W-2): template_utility_ranked rows emit at the try_execute
  // gate-threshold seam + the CLI. Renders smoothed utility vs raw
  // first_try_ratio side-by-side (the F-7 before/after view).
  import TemplateRankingDrawer from '../drawers/TemplateRankingDrawer.svelte';
  // F-13 / Slice A5 — SessionArchiveDrawer (DrawerKey session-archive).
  // Producer gate ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE is default-OFF / wire-dark
  // (forensic opt-in): session_archived rows emit at the run_migration DROP site
  // when a stale-version SSB is cold-archived before the destructive DROP.
  import SessionArchiveDrawer from '../drawers/SessionArchiveDrawer.svelte';
  // G-4 / Slice A — McpRugPullDrawer (DrawerKey mcp-rug-pull).
  // Default-ON (W-2): mcp_rug_pull_blocked rows emit from the live MCP bridge
  // re-pin gate (DefaultMcpBridge::http_list_tools) when a reconnect re-fetch
  // from a previously-approved server no longer matches the blake3 schema pin.
  import McpRugPullDrawer from '../drawers/McpRugPullDrawer.svelte';
  // G-1 / Slice A — TaintFlowDrawer (DrawerKey taint-flow).
  // Default-ON (W-2): taint_flow rows emit from the live data-plane taint gate
  // (production_agent_rpc git_commit / edit arms) when an untrusted ingest
  // reaches a destructive sink and the capability gate blocks it.
  import TaintFlowDrawer from '../drawers/TaintFlowDrawer.svelte';
  // G-2 / Slice A — RraViewerPanel (DrawerKey rra-viewer).
  // Default-ON (W-2): run_artifact_emitted rows emit at the `elai run` exit when
  // a Reproducible-Run Artifact (.rra) bundle is sealed. The only opt-out is the
  // kill-switch ELAI_RRA_DISABLE. Panel lives under windows/ per the G-2
  // WIRING_CONTRACT; mounted here as a drawer key so it has a wired route.
  import RraViewerPanel from '../windows/RraViewerPanel.svelte';
  // G-3 / Slice A — BlastRadiusDrawer (DrawerKey blast-radius).
  // Default-ON (W-2): blast_radius_blocked rows emit from the deterministic
  // runtime gates (translate_tool_request / run_bash FS-escape arm +
  // fetch_and_extract egress arm) when a fail-closed gate refuses a
  // post-expansion FS escape or a denied egress host. The only opt-out is the
  // kill-switch ELAI_BLAST_RADIUS_DISABLE.
  import BlastRadiusDrawer from '../drawers/BlastRadiusDrawer.svelte';
  // G-8 / Slice A — AttestationDrawer (DrawerKey attestation).
  // Default-ON (W-2): provenance_attestation_emitted rows emit at the `elai run`
  // exit when a signed in-toto v1 provenance attestation is built from the sealed
  // G-2 Reproducible-Run Artifact + signed with a LOCAL blake3-Merkle proof
  // (re-verifiable OFFLINE). The only opt-out is the kill-switch
  // ELAI_ATTESTATION_DISABLE.
  import AttestationDrawer from '../drawers/AttestationDrawer.svelte';
  // G-9 / Slice A — RedactionPanel (DrawerKey redaction).
  // Default-ON (W-2): secret_redacted advisory rows emit whenever the write-time
  // secret+PII redactor fires on a typed SsbPayload free-text field. The row
  // carries the redacted field + stable blake3-keyed token(s) + match count,
  // NEVER the raw secret. Redaction is always on (no enable flag).
  import RedactionPanel from '../drawers/RedactionPanel.svelte';
  // G-7 / Slice A — ConfidenceDrawer (DrawerKey confidence).
  // claim_confidence_scored rows carry the REAL fused per-change confidence
  // feeding the conformal decision (producer behind the conformal triple gate);
  // trifecta_quarantined rows are the lethal-trifecta dual-LLM quarantine beacon
  // (default-ON, kill-switch ELAI_TRIFECTA_QUARANTINE_DISABLE only).
  import ConfidenceDrawer from '../drawers/ConfidenceDrawer.svelte';
  // G-6 / SCHEMA_VERSION 217 — SafeDeltaPanel (DrawerKey safe-delta).
  // safety_delta_measured rows emitted by `elai bench safedelta`. Surfaces the
  // derived SAFEDELTA (gate-ON-vs-OFF marginal residual rate) + 95% bootstrap
  // CI + corpus_blake3 + gate_recall + runpod_pending status.
  // The bench command IS the entrypoint (W-2, no ELAI_*_ENABLE gate; Tier-2
  // dynamic arm is RunPod-gated). cross-lang-pair: dashboard/tauri/api.
  import SafeDeltaPanel from '../drawers/SafeDeltaPanel.svelte';
  // G-10 / SCHEMA_VERSION 218 — ModelDriftPanel (DrawerKey model-drift).
  // model_drift_suspected rows emitted at `elai run` exit or `elai model-drift check`.
  // Surfaces identity_key + drift kind + signals + per_family_delta +
  // affected_template_ids + confirmed_by_fingerprint (GD-5 honest badge).
  // Default-ON (W-2); ELAI_MODEL_DRIFT_DISABLE kill-switch only.
  // cross-lang-pair: dashboard/tauri/api.
  import ModelDriftPanel from '../drawers/ModelDriftPanel.svelte';
  import PagerankPanel from '../drawers/PagerankPanel.svelte';

  let { drawerKey }: { drawerKey: DrawerKey } = $props();
</script>

{#if drawerKey === 'knowledge'}<KnowledgeDrawer />{:else if drawerKey === 'reasoning'}<ReasoningDrawer />{:else if drawerKey === 'orchestration'}<OrchestrationDrawer />{:else if drawerKey === 'rules'}<RulesDrawer />{:else if drawerKey === 'research'}<ResearchDrawer />{:else if drawerKey === 'providers'}<ProvidersDrawer />{:else if drawerKey === 'project-intelligence'}<ProjectIntelligenceDrawer />{:else if drawerKey === 'anti-staleness'}<AntiStalenessDrawer />{:else if drawerKey === 'tools-hooks'}<ToolsHooksDrawer />{:else if drawerKey === 'web-quality'}<WebQualityDrawer />{:else if drawerKey === 'cosmos'}<CosmosDrawer />{:else if drawerKey === 'sessions'}<SessionsDrawer />{:else if drawerKey === 'mutation-archive'}<MutationArchivePanel />{:else if drawerKey === 'ppm-scores'}<PpmScoresPanel />{:else if drawerKey === 'verification-consensus'}<VerificationConsensusPanel />{:else if drawerKey === 'tripwires'}<TripwirePanel />{:else if drawerKey === 'incidents'}<IncidentsDrawer />{:else if drawerKey === 'prmbench'}<PrmBenchPanel />{:else if drawerKey === 'sycophancy-bench'}<SycophancyBenchPanel />{:else if drawerKey === 'bai-arms'}<BaiArmsPanel />{:else if drawerKey === 'held-out'}<HeldOutPanel />{:else if drawerKey === 'canary'}<CanaryPanel />{:else if drawerKey === 'locked-keys'}<LockedKeysPanel />{:else if drawerKey === 'mode-routing'}<ModeRoutingPanel />{:else if drawerKey === 'mode-routing-bench'}<ModeRoutingBenchPanel />{:else if drawerKey === 'checker-registry'}<CheckerRegistryPanel />{:else if drawerKey === 'adversary'}<AdversaryPanel />{:else if drawerKey === 'baby-step'}<BabyStepPanel />{:else if drawerKey === 'conformal-calibration'}<ConformalCalibrationPanel />{:else if drawerKey === 'conformal-validation'}<ConformalValidationPanel />{:else if drawerKey === 'novelty'}<NoveltyPanel />{:else if drawerKey === 'observable-effect'}<ObservableEffectPanel />{:else if drawerKey === 'refusal-sla'}<RefusalRateGauge />{:else if drawerKey === 'arc-bench'}<ARCBenchPanel />{:else if drawerKey === 'planning-ledger'}<PlanningLedgerPanel />{:else if drawerKey === 'memory-facts'}<MemoryFactsDrawer />{:else if drawerKey === 'chat-memory'}<MemoryNotesPanel />{:else if drawerKey === 'grammar-compile'}<GrammarCompilePanel />{:else if drawerKey === 'archive-explorer'}<ArchiveExplorerView />{:else if drawerKey === 'discovery-queue'}<DiscoveryQueueDrawer />{:else if drawerKey === 'mode-routing-dashboard'}<ModeRoutingDashboardView />{:else if drawerKey === 'failure-modes'}<FailureModesDrawer />{:else if drawerKey === 'spend'}<SpendDrawer />{:else if drawerKey === 'query'}<QueryDrawer />{:else if drawerKey === 'transcript'}<TranscriptDrawer />{:else if drawerKey === 'skills'}<SkillsPanel />{:else if drawerKey === 'vibegate'}<VibeGateDrawer />{:else if drawerKey === 'security-rules'}<SecurityRulesPanel />{:else if drawerKey === 'deps-security'}<DepsSecurityPanel />{:else if drawerKey === 'security-monotonicity'}<SecurityMonotonicityDrawer />{:else if drawerKey === 'staleness-quarantine'}<StalenessQuarantineDrawer />{:else if drawerKey === 'template-ranking'}<TemplateRankingDrawer />{:else if drawerKey === 'session-archive'}<SessionArchiveDrawer />{:else if drawerKey === 'mcp-rug-pull'}<McpRugPullDrawer />{:else if drawerKey === 'taint-flow'}<TaintFlowDrawer />{:else if drawerKey === 'rra-viewer'}<RraViewerPanel />{:else if drawerKey === 'blast-radius'}<BlastRadiusDrawer />{:else if drawerKey === 'attestation'}<AttestationDrawer />{:else if drawerKey === 'redaction'}<RedactionPanel />{:else if drawerKey === 'confidence'}<ConfidenceDrawer />{:else if drawerKey === 'safe-delta'}<SafeDeltaPanel />{:else if drawerKey === 'model-drift'}<ModelDriftPanel />{:else if drawerKey === 'pagerank'}<PagerankPanel />{:else if drawerKey === 'decomposition-entropy'}<DecompositionEntropyHistogram />{/if}
