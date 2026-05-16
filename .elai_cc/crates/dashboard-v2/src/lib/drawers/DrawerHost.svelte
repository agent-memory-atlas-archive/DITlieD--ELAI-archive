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

  let { drawerKey }: { drawerKey: DrawerKey } = $props();
</script>

{#if drawerKey === 'knowledge'}<KnowledgeDrawer />{:else if drawerKey === 'reasoning'}<ReasoningDrawer />{:else if drawerKey === 'orchestration'}<OrchestrationDrawer />{:else if drawerKey === 'rules'}<RulesDrawer />{:else if drawerKey === 'research'}<ResearchDrawer />{:else if drawerKey === 'providers'}<ProvidersDrawer />{:else if drawerKey === 'project-intelligence'}<ProjectIntelligenceDrawer />{:else if drawerKey === 'anti-staleness'}<AntiStalenessDrawer />{:else if drawerKey === 'tools-hooks'}<ToolsHooksDrawer />{:else if drawerKey === 'web-quality'}<WebQualityDrawer />{:else if drawerKey === 'cosmos'}<CosmosDrawer />{/if}
