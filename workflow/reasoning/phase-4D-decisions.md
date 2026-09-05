Phase 4D.1-4D.6 Decision Doc

What: research_tree.rs skeleton with all FlashResearch types, GRC config update for Researcher (10/1200), TDD red phase, adversarial tests, full implementation, metrics.

Key decisions:
1. SSB: Add ResearchStep variant to SsbActionType enum + payload + from_str + action_type match arms. No schema migration needed (CHECK constraint is code-level, not SQL migration).
2. NaN safety: Manual PartialEq impl on ResearchNode (no derive) since phi/psi are f64. ordered_float not in deps, not adding it.
3. Bundleable items assessment:
   - [PM] SurgicalDecomposer/blast radius/project detection: DEFER (needs pipeline-modes-design.txt, unread)
   - [PM] Oracle mode/source tiers: DEFER (overlaps with A-43/A-45/A-47, needs verification)
   - [TE] ModelCapabilityProfile depth adaptation: DEFER (needs token-efficiency-design.txt)
   - [CC] FILE_PLAN as 9th BabyStepSpec: DEFER (needs code-completeness-enforcement-design.txt)
   - [RA] COMPLEXITY_HIGH/AB-MCTS: DEFER (needs reasoning-architecture-design.txt)
   Result: zero bundleable items included. All require unread design docs.
4. GrcEngine::apply_agent_config does NOT exist. Will set GRC config via default_grc_config match arm only.
5. SourceQuality domain lookup: exact match via host_str() == "arxiv.org" or ends_with(".arxiv.org"), never substring.
6. Speculative depth: forced to 0 when provider is local. Stored as field, checked at tree construction.

Expected test outcomes: red at 4D.3 (stubs return errors/defaults), green at 4D.5 (logic filled in).
Unknowns: None remaining after pre-gate grep sweep.
