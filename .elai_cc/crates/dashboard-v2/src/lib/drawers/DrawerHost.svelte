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
  // C-41 — Application Skills System inspector. Mounted as the 6th
  // SettingsWindow inspector card; also surfaced here so the DrawerKey
  // union stays exhaustive over DrawerHost's switch.
  import SkillsPanel from '../components/SkillsPanel.svelte';

  let { drawerKey }: { drawerKey: DrawerKey } = $props();
</script>

{#if drawerKey === 'knowledge'}<KnowledgeDrawer />{:else if drawerKey === 'reasoning'}<ReasoningDrawer />{:else if drawerKey === 'orchestration'}<OrchestrationDrawer />{:else if drawerKey === 'rules'}<RulesDrawer />{:else if drawerKey === 'research'}<ResearchDrawer />{:else if drawerKey === 'providers'}<ProvidersDrawer />{:else if drawerKey === 'project-intelligence'}<ProjectIntelligenceDrawer />{:else if drawerKey === 'anti-staleness'}<AntiStalenessDrawer />{:else if drawerKey === 'tools-hooks'}<ToolsHooksDrawer />{:else if drawerKey === 'web-quality'}<WebQualityDrawer />{:else if drawerKey === 'cosmos'}<CosmosDrawer />{:else if drawerKey === 'sessions'}<SessionsDrawer />{:else if drawerKey === 'mutation-archive'}<MutationArchivePanel />{:else if drawerKey === 'ppm-scores'}<PpmScoresPanel />{:else if drawerKey === 'verification-consensus'}<VerificationConsensusPanel />{:else if drawerKey === 'tripwires'}<TripwirePanel />{:else if drawerKey === 'incidents'}<IncidentsDrawer />{:else if drawerKey === 'prmbench'}<PrmBenchPanel />{:else if drawerKey === 'sycophancy-bench'}<SycophancyBenchPanel />{:else if drawerKey === 'bai-arms'}<BaiArmsPanel />{:else if drawerKey === 'held-out'}<HeldOutPanel />{:else if drawerKey === 'canary'}<CanaryPanel />{:else if drawerKey === 'locked-keys'}<LockedKeysPanel />{:else if drawerKey === 'mode-routing'}<ModeRoutingPanel />{:else if drawerKey === 'mode-routing-bench'}<ModeRoutingBenchPanel />{:else if drawerKey === 'checker-registry'}<CheckerRegistryPanel />{:else if drawerKey === 'adversary'}<AdversaryPanel />{:else if drawerKey === 'baby-step'}<BabyStepPanel />{:else if drawerKey === 'conformal-calibration'}<ConformalCalibrationPanel />{:else if drawerKey === 'conformal-validation'}<ConformalValidationPanel />{:else if drawerKey === 'novelty'}<NoveltyPanel />{:else if drawerKey === 'observable-effect'}<ObservableEffectPanel />{:else if drawerKey === 'refusal-sla'}<RefusalRateGauge />{:else if drawerKey === 'arc-bench'}<ARCBenchPanel />{:else if drawerKey === 'planning-ledger'}<PlanningLedgerPanel />{:else if drawerKey === 'memory-facts'}<MemoryFactsDrawer />{:else if drawerKey === 'chat-memory'}<MemoryNotesPanel />{:else if drawerKey === 'grammar-compile'}<GrammarCompilePanel />{:else if drawerKey === 'archive-explorer'}<ArchiveExplorerView />{:else if drawerKey === 'discovery-queue'}<DiscoveryQueueDrawer />{:else if drawerKey === 'mode-routing-dashboard'}<ModeRoutingDashboardView />{:else if drawerKey === 'failure-modes'}<FailureModesDrawer />{:else if drawerKey === 'skills'}<SkillsPanel />{/if}
