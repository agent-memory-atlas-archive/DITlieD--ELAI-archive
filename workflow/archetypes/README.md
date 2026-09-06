workflow/archetypes/

E-11 Tier-A scaffolding directory. Holds SaaS-archetype templates + scope-clarity
checklists + production-grade scaffolding metadata. Mirrors the workflow/vision_rag/
pre-E-36 pattern: directory created by Tier-A; populated by Tier-B.

CONTEXT

E-11 (idea-to-SaaS scope clarity) builds on the shipped B-40 Idea-Framer:
  - FsmSubstate::Framing (orchestrator/src/idea_framing/)
  - FramedIdea + CriticRound + CriticFinding types
  - IdeaFramingWindow.svelte
  - 5 SSB variants (IdeaFramingStarted / ProjectClassClassified /
    CriticRoundComplete / IdeaFramed / TradeoffCardEmitted) at SCHEMA 41
  - AgentRole::IdeaFramer + AgentRole::IdeaCritic
  - 2 Tauri commands (per project_b40_shipped memory entry)

E-11 adds the "scope clarity" + "production-grade vs demo-grade" semantics.
Tier-A this run shipped 4 negative-space contract guards (see
crates/contracts/tests/e11_idea_to_saas_contracts.rs). Tier-B will populate
this directory with the archetype templates themselves.

TIER-B SCHEDULE (deferred from s-orch-e11 per workflow/reasoning/E-11-decisions.md)

  templates/         — per-archetype JSON / TOML templates (auth, rate-limit,
                       migration plan, observability budget, error contract).
                       Archetypes: SaaSDashboard / B2BMarketplace /
                       InternalTooling / DeveloperAPI / ConsumerSocial.
  checklists/        — 15-item production-grade checklist per archetype.
                       Drives AuditorMode::ProductionGrade verdicts.
  prior_art/         — cited prior-art tools (v0.dev / bolt.new / replit /
                       cursor / lovable / horizon) with the gap analysis
                       that motivates ELAI's archetype-driven path.
  decisions/         — per-archetype design decisions (auth scheme choice,
                       multi-tenancy isolation level, etc).

PATTERN

Mirrors workflow/vision_rag/ (E-36) + workflow/compliance/ (E-38 deferred).
Each Tier-A run creates an empty README scaffold; Tier-B fills in concrete
content once the producer wiring lands.

END
