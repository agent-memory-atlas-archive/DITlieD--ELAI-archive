GROUNDING SWEEP -- phase H-31
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-31_user_decision_bitemporal_ledger.md
Date: 2026-06-06
FSM: PLANNING | Session: s-8404 lineage

SPEC REFERENCES
No explicit "Spec references:" or "## Reads (mandatory)" blocks. Grounding is via inline "## Grounding" section (line 84) + ELAI file:line citations (line 92). Treated as equivalent.

arXiv:2501.13956 (Zep/Graphiti) | GROUNDED | zep-graphiti-2501.13956-interface.md verified: "Zep: A Temporal Knowledge Graph Architecture for Agent Memory" by Rasmussen et al. Bi-temporal soft-invalidation (t_invalid set to t_valid of superseding fact) confirmed. +68% token narration number NOT in this paper -- H-31 correctly does NOT cite that number (H-30 did; already flagged there).

arXiv:2504.19413 (Mem0 pipeline) | INGEST | No interface file in workflow/papers/. Not in RAG. Plan cites ADD/UPDATE/DELETE/NOOP design and DELETE-BANNED-for-replay claim. Cannot confirm ID resolves to a Mem0 paper without ingest.

arXiv:2505.07509 (HALO per-class decay) | INGEST | No interface file. Not in RAG. Plan cites per-class half-life decay. Cannot confirm.

arXiv:2510.09905 ("Personalization-Trap") | UNGROUNDED -- SEE ACTIONS | No interface file. Not in RAG. Not found in any paper search path. ELAI has a documented history of fabricated arXiv IDs (H-30 sweep caught +68% Zep fabrication; B-40/B-42 had 2 bad cites per auto-memory). This ID cannot be confirmed from local knowledge. The claim (persona/user-memory injection biases reviewer toward user) is sound engineering reasoning, but the CITATION is unverified.

arXiv:2605.06527 (staleness reference) | UNGROUNDED | Plan itself flags this: "in the future relative to the 2026-01 knowledge cutoff and could NOT be verified this session". Self-classified as unverified by the plan author.

odysseus (MIT prior-art, docs/.source/odysseus-main/) | VERIFY_NEEDED | Plan cites memory.py:291, memory.py:255, memory_vector.py:116, memory_extractor.py:535-540, context_budget.py:21. Glob confirms odysseus-main directory exists (odysseus-INDEX.md referenced in auto-memory). Line numbers not independently verified here. Treat as HIGH CONFIDENCE (source ingested this session per H-plan notes) but flag for executor to re-confirm line numbers at authoring time.

ELAI code refs (fact.rs, store.rs, mem0_dispatcher.rs, retrieval.rs, decay.rs, context_budget.rs, live_cli.rs, ssb.rs, action_type_lock.rs) | GROUNDED with SCHEMA MISMATCH -- see below

CLAIM CLASSIFICATION

FactType has 8 variants incl. Preference at fact.rs:35 | GROUNDED | fact.rs:20-43 read directly: 8 variants confirmed; Preference at line 35 confirmed; retrieval threshold Preference=0.5, Opinion=0.7 confirmed at fact.rs:81-92.

store.rs:208 supersede atomic tx soft-close | GROUNDED | store.rs:208-260 read: UPDATE sets valid_to_ms=now_ms+superseded_by, then INSERT new row. DELETE never issued. Matches plan exactly.

store.rs:303 soft_close WITHOUT replacement | GROUNDED | store.rs:303-313 read: UPDATE valid_to_ms=now_ms WHERE valid_to_ms IS NULL. NotFound if already closed. Matches plan.

store.rs:147 list_current valid_to_ms IS NULL filter | GROUNDED | store.rs:147-161 read: WHERE valid_to_ms IS NULL confirmed.

mem0_dispatcher dispatch_one/dispatch_batch WIRE-DARK | GROUNDED | mem0_dispatcher.rs:144-228 read; test block at :337-378 confirmed only #[cfg(test)] callers. No production callers seen.

retrieval.rs:46 get_relevant_facts WIRE-DARK | GROUNDED | retrieval.rs:46 read; plan states "every caller is a test (elai.map CALL_RESOLVED 281701-281705 all land in tests/)". Code confirms wire-dark: conversation.rs grep returned zero matches for get_relevant_facts.

ContextSlot has 8 variants, NO Memory slot | GROUNDED | context_budget.rs:1066-1074 read: 8 variants confirmed (SystemPrompt=0..OutputReserve=7), NO Memory variant. Slot 2 is PhaseSpec (NOT "Telescope" -- plan correctly notes this discrepancy at line 28). SLOT_COUNT=8 at :10 confirmed.

conversation.rs has ZERO references to elai_memory/get_relevant_facts | GROUNDED | grep returned no matches.

SCHEMA_VERSION=220 / TOTAL_VARIANTS=497 | MISMATCH | LIVE values: SCHEMA_VERSION=239 (ssb.rs:3271), TOTAL_VARIANTS=516 (action_type_lock.rs:817). Plan says 220/497 and labels CLAUDE.md's 191/464 as STALE; the plan's own 220/497 is now also stale by 19 schema versions and 19 variants. The bump math "+1 to 221/498" is wrong against live baseline -- correct bump is "239->240 / 516->517".

live_cli.rs audit_conflicts at :2034 + adjudicate_semantic at :2076 PROD-WIRED | GROUNDED | live_cli.rs grep shows audit_conflicts called at :2096, adjudicate_semantic at :2138. Line offsets shifted (~60 lines) from plan's :2034/:2076 but the calls ARE present and PROD-WIRED.

decay.rs:29 default_decay_rate per FactType | GROUNDED | decay.rs:29-39 read: per-FactType f32 rates confirmed (Preference=0.001, Opinion=0.0005 etc.).

Mem0 ADD/UPDATE/DELETE/NOOP + DELETE BANNED for replay | INGEST | Relies on arXiv:2504.19413 not yet ingested; the delete-banned claim is corroborated by ELAI's own soft_close design (code-grounded) but the paper citation needs ingest.

HALO per-class half-life is the backstop | INGEST | Relies on arXiv:2505.07509 not yet ingested; decay.rs code confirms per-class half-life math exists, but paper grounding absent.

Personalization-Trap auditor firewall | UNGROUNDED (citation) | The engineering decision (force-empty Memory slot for auditor/critic/adversary roles) is sound and internally consistent with ELAI's trust product design. The specific CITATION arXiv:2510.09905 is unverified -- no interface file, not in RAG, cannot confirm this ID maps to a personalization-bias paper vs. an unrelated paper or a fabricated ID.

+68% token overhead from narrating superseded rows | NOT CITED in H-31 | H-31 correctly avoids citing this number (it was the H-30 fabrication catch). The plan references the Zep 115K->1.6K token reduction (verified) as rationale for SQL-exclusion. No fabrication risk here.

arXiv:2605.06527 staleness reference | UNGROUNDED | Future-dated relative to training cutoff; plan self-flags. No numbers from this citation should be used.

ACTIONS REQUIRED
- MISMATCH: plan claims SCHEMA_VERSION=220 / TOTAL_VARIANTS=497 as the baseline; live values are 239/516. The "+1 bump to 221/498" math in Integration section is wrong. Executor MUST read ssb.rs and action_type_lock.rs at authoring time and use live values (currently 239/516 -> bump to 240/517).
- INGEST: arXiv:2510.09905 -- spawn paper-ingestor mode=paper with id=2510.09905. Confirm the paper title and that it measures persona/preference injection biasing a judge/reviewer/adversary. If the ID does not resolve to that finding, the auditor-objectivity firewall citation must be relabeled "Ungrounded conjecture:" in the plan or replaced with a verified cite. The ENGINEERING DECISION itself is sound regardless; only the citation is at risk.
- INGEST: arXiv:2504.19413 -- spawn paper-ingestor mode=paper with id=2504.19413. Confirm this is a Mem0 paper covering ADD/UPDATE/DELETE/NOOP pipeline and the delete-for-replay design.
- INGEST: arXiv:2505.07509 -- spawn paper-ingestor mode=paper with id=2505.07509. Confirm HALO per-class half-life decay.
- UNGROUNDED: arXiv:2605.06527 -- plan already self-flags this. Remove or label "Ungrounded: future-dated, cannot verify" before TESTING_FIRST. Do not present any numbers from this cite.
- VERIFY at authoring: odysseus line numbers (memory.py:291/255, memory_vector.py:116, memory_extractor.py:535-540, context_budget.py:21) -- plan author noted these were "confirmed this session (W-7)" but executor should re-confirm at authoring time against the local copy at docs/.source/odysseus-main/.

VALIDATION WARNINGS
- No explicit "Spec references:" lines in source -- plan uses inline "## Grounding" section instead. Acceptable for an H-plan activation unit; spec-grounder treated it as equivalent.
- SCHEMA_VERSION and TOTAL_VARIANTS in plan are stale by 19 versions. This is an EXECUTING-time correctness risk (wrong bump math), not a design risk.
- arXiv:2510.09905 has ZERO evidence of prior ingest anywhere in this workflow. Given ELAI's documented fabrication history on arXiv IDs, this must be verified before TESTING_FIRST.
- arXiv:2605.06527 is explicitly self-flagged as unverified in the plan. It must be removed or replaced before TESTING_FIRST.
