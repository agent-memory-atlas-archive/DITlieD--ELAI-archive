Tranche-D-hygiene-decisions.md

Session: s-22300 (Tranche D plan-doc hygiene)
Date: 2026-05-31
Scope: stale-contract-test SCHEMA-pin sweep + E-50 vocab refresh + Theme-1 Pattern-2 top-5 + Theme-6 path drift + B-0_INDEX status flips + B-3/B-10/C-15 cosmetic
Posture: cosmetic-only edits (no source/runtime delta); no cargo run; no commit; no new deps
Source explorer: workflow/sessions/s-22300/exploration/tranche-D-hygiene.md (8 categories, drift findings)
Source context: workflow/sessions/s-22300/context-package-tranche-D-hygiene.md

DECISIONS

DECISION-1: SCHEMA-pin floor sweep (D-1) SHIPPED across 9 contract test files.
Rationale: assert_eq!(SCHEMA_VERSION_PUBLIC, N, ...) blocks legitimate forward bumps; ceiling-pin is wrong primitive when SCHEMA_VERSION monotonically grows. Replaced with assert!(SCHEMA_VERSION_PUBLIC >= N, "schema only drifts forward; this contract was written at version N, ..."). 8 files per brief + the B-7 e13_e12_tdd_verification_contract.rs (consistency — pinned at 195 exact, also converted to floor).
Files: activity_ribbon_ssb_contract.rs (103), b6_quad_ssb_contract.rs (194), e38_ai_roadmap_signals_ssb_contract.rs (175), e40_symbol_graph_contract.rs (193), e47_lora_ship_gate_contract.rs (192), e54_eval_corpus_ssb_contract.rs (167), idea_framing_contract.rs (96), minified_chunked_ssb_contract.rs (96), e13_e12_tdd_verification_contract.rs (195).
Comment-block preservation: idea_framing_contract.rs L74-76 (s-68264 ledger comment) PRESERVED; minified_chunked_ssb_contract.rs L71-76 (B-33/B-34 history comment) PRESERVED — assertion body edited in-place, surrounding comments left intact verbatim.
Grounding: user feedback "schema only drifts forward" + B-7 Tranche audit precedent + Tranche-B7 retry confirmed a 9th pin should NOT be added at 195.

DECISION-2: E-50 vocab refresh (D-2) SHIPPED 8 mechanical line edits + 1 dictionary-block prelude.
Rationale: E-50 plan body was authored with legacy 4-bucket taxonomy (B2C/B2B/API-first/AI-app) but canonical surface in .elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs:33-67 ships 5-variant enum (SaasDashboard / B2bMarketplace / InternalTooling / DeveloperApi / ConsumerSocial). Map is NOT 1:1 — B2B forks on single-tenant (SaasDashboard) vs two-sided (B2bMarketplace); B2C forks on dashboard vs ConsumerSocial; AI-app collapses into matching product shape (no dedicated AiApp root). Solution: prepend a "Vocabulary dictionary" block at top-of-file documenting the legacy -> canonical map before doing per-line replacements, so future readers see the lineage + the fork rules. Without the dictionary block, naive find-replace would mis-classify two-sided B2B marketplaces as single-tenant dashboards.
Edits: L1 (title) / L10 (Context prose) / L18 (E-11 archetype-vendor mapping) / L20 (multi_tenant default) / L47 (Task 1 enum spec) / L67 (Task 11 catalog stems) / L74 (WIRING_CONTRACT produces_symbols + ALL_SAAS_ARCHETYPES const added) / L117 (observable_effect expect_artifact B2B->SaasDashboard) / L125 (DETERMINISTIC_CHECK rg patterns 4->5 variants + TOML stem update).
TOML stems: legacy {b2c, b2b, api_first, ai_app}.toml -> canonical {saas_dashboard, b2b_marketplace, internal_tooling, developer_api, consumer_social}.toml matching as_snake_case impl at saas_archetype.rs:73-82.

DECISION-3: Theme-1 Pattern-2 vocab refresh (D-3) — E-31 SHIPPED 3 line edits; other 4 plans SKIPPED.
Rationale: exploration found that 4 of 5 Theme-1 top-5 plans (E-43 / E-19 / E-39 / E-53) have ZERO legacy "atomic +N variants" vocabulary in their plan bodies. Context-package's top-5 inference was unreliable (admitted not ordered). Only E-31 has 3 hits at L66 (CloudRouteDecision "new variant"), L107 (rg pattern), L153 (MODIFY ssb.rs "two new SsbActionType variants"). Edits replace stale "new SsbActionType variant" vocabulary with canonical "Pattern-2 43rd application: single-variant SsbActionType::RouteDecisionEvent + 3-arm RouteDecisionEventKind discriminator (CloudRouteChosen / TierModelSwapped / LocalRouteKept)" pattern per CLAUDE.md E-31 tier-matrix block + Theme-1 MASTER-INDEX precedent.
Other 4 plans (E-43 / E-19 / E-39 / E-53): SKIPPED. Rejected the NOTE-block append alternative — plan bodies do not contain legacy vocab to refresh, so appending a clarifying NOTE adds noise without correcting any drift. Future selection of Theme-1 top-5 should be by Grep-density not citation-density.

DECISION-4: Theme-6 plan path drift (D-4) DEFERRED — 8 plans need per-plan HUMAN-REVIEW.
Rationale: all 8 canonical target paths verified on disk by explorer (C-30 production_agent_rpc.rs / C-40 cli/src/github_pr_cmd.rs / C-45 credit_attribution/integration.rs / E-20 cost/cloud_route.rs / E-21 peer_systems/mod.rs / E-25 architecture/claude_code_arch.rs / E-38 compliance.rs / E-46 planning_decomposition/mod.rs). BUT per-plan line-resolution Grep+Edit step required — stale-path tokens (tool_dispatcher / crates/github_adapter / agent/dropper / orchestrator/routing / auditor/ / safety/eu_ai_act_compliance) were not pre-resolved to lines in the explorer to stay under tranche scope. Each plan needs one Grep pass per stale token then mechanical Edit. Deferred to a follow-up hygiene session. Logged in stubs.md as TrancheD-Theme6-path-drift.

DECISION-5: B-0_INDEX status flips (D-5) ABORTED — file structure is session-narrative, not row-per-plan.
Rationale: context-package premise of "12 plan-status row updates" + "B-0_INDEX.md 14000+ lines too big to read" both wrong. Explorer found B-0_INDEX.md is 1015 lines structured as 5 SESSION blocks (L89, L244, L294, L333, L385). Per-plan status appears INSIDE session blocks as prose "B-N DONE/SHIPPED/PARTIAL" lines, NOT as row-per-plan rows in a status table. Grep confirmed: 11 of 12 plan codes (C-42/C-47/C-50/D-18/E-12/E-13/E-25/E-40/E-47/E-50/E-52) return ZERO hits. Only B-39 has existing prose at L318/L362 already reflecting SHIPPED. Mechanical per-row Edit would fail (no row exists) or worse create orphan content not matching the file's narrative pattern.
Future strategy (logged in stubs.md as TrancheD-B0INDEX-status-flips): APPEND a new "SESSION s-XXXXX -- PLAN-STATUS RECONCILIATION (YYYY-MM-DD)" block at end-of-file listing the 12 verdicts as new prose entries. This matches the file's append-only narrative pattern. Requires a future session to author the block.

DECISION-6: B-3 / B-10 / C-15 documentation drift (cosmetic) DEFERRED.
Rationale: per-plan line-offset audit needs Grep verification step per plan that the explorer did not pre-resolve. B-3 needs current stub.ts export count (explorer hit ~45 stub-related lines but "export" count requires distinct regex). B-10 line offsets confidently drifted (1842/6448/6565/6759/7513 -> 2751/15609/15743/15948/16726 per explorer) but L4059 outlier (appcontainer_profile_id) needs verification before declaring 5 vs 6 canonical sites. C-15 clock.rs line 33 -> 43 and rustc-hash Cargo.toml line 46 -> 59 known drifted; closure-marker block at L29+ must be PRESERVED verbatim. Each plan is small per-edit but needs an additional verification pass per plan. Deferred to a future hygiene session. Logged in stubs.md as TrancheD-docdrift-B3-B10-C15.

NET TRANCHE-D FOOTPRINT (POST-DRIFT-CORRECTION)

Mechanical edits shipped this tranche:
- 9 contract-test floor swaps (8 brief + 1 consistency)
- 8 E-50 line edits + 1 dictionary-block prelude (12 logical edits total)
- 3 E-31 vocab refreshes
Total: ~21 mechanical edits across 11 files

Deferred to future sessions (logged as stubs):
- 8 Theme-6 path drift (TrancheD-Theme6-path-drift)
- B-0_INDEX status reconciliation (TrancheD-B0INDEX-status-flips, append-session-block strategy)
- B-3 + B-10 + C-15 documentation drift (TrancheD-docdrift-B3-B10-C15)

Aborted as scope-mismatch (not deferred — premise contradicted by source):
- 11 of 12 B-0_INDEX row edits (no row exists; file is session-narrative)
- 4 of 5 Theme-1 top-5 vocab refreshes (E-43 / E-19 / E-39 / E-53 have zero legacy vocab; refresh would be no-op)

POSTURE NOTES

No cargo / cargo check / svelte-check / cargo test invoked this tranche per user direction (cosmetic-only edits; auditor runs build verification).
No commit / push (Tranche-D is one chunk in a larger session; per CLAUDE.md commit-cadence ONE commit per FULL session, not per tranche).
No new dependencies added.
No source code under .elai_cc/crates/ touched apart from the 9 SCHEMA-pin floor-pattern swaps (mechanical, single-line assertion shape change, no semantic delta).

GROUNDING TRAIL

Context-package: workflow/sessions/s-22300/context-package-tranche-D-hygiene.md (8 categories, 6 VALIDATION-WARNING flags)
Explorer: workflow/sessions/s-22300/exploration/tranche-D-hygiene.md (8 categories, 6 DRIFT findings vs context-package, confidence 0.78)
Canonical taxonomy: .elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs lines 33-67 (enum + ALL_SAAS_ARCHETYPES const) + 73-96 (as_snake_case / from_snake_case impls)
Pattern-2 precedent: workflow/sessions/s-night-2026-05-30/audit/MASTER-INDEX-NIGHT-2026-05-30.md Theme-1 (~32 plans cited)
Schema-floor pattern: user feedback "schema only drifts forward" + Tranche-B7 retry precedent (ssb.rs SCHEMA_VERSION_PUBLIC at 195 currently; all 9 contracts now assert >= N not == N)
