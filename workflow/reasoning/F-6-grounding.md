GROUNDING SWEEP -- phase F-6
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-6_learned_skill_library.md
Date: 2026-06-02
FSM: PLANNING | Session: s-60824

SCOPE NOTE: This sweep targets the three external-behavior claims the dispatch named, plus
every NUMBER in F-6 that must NOT become a release target. F-6 itself makes NO external
performance claim (it borrows methodology, defers the only quantitative measurable). The danger
in F-6 is the inverse of a normal grounding sweep: not unverified claims being acted on, but
a non-transferable Minecraft number, a synthetic SHARD number, or a not-yet-baselined ELAI
number silently becoming an acceptance gate. All three named numbers are flagged NON-TARGET.

SPEC REFERENCES
(F-6 cites synthesis + research dossiers, not docs/specs/guide/steps/phase-*.txt. The "Reads
(mandatory)" block points at real crate source + the synthesis doc, all verified below.)
VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 C6 / §1.5 / §3 Pillar 3 / §7 | GROUNDED | file read in full; §1.5 (Voyager 3.3x non-transferable), §4 C6 (REVISED SCOPE C6a/b/c + "Defer the 70%-reuse measurable"), §7 (4 "Do NOT" C6 clauses) all present at lines 104-106, 363-387, 497-504.
00-COMPARISON-shard-vs-priorart.md Pillar 3 | GROUNDED | lines 53-59 + 82-83 + 92: T1-T4 ladder is shipped real code (IntEnum), "T4 tier never exercised (30-day gate)" confirms min-days + human-gated top tier.
05-voyager.md (Voyager dossier) | GROUNDED | section 5 (lines 120-127) + citation [1]/[12]: 3.3x is "3.3x more unique items collected vs prior SOTA", Minecraft tech-tree, VENDOR/AUTHOR-REPORTED, code frozen 2024-04-03, single-digit seeds.
crates/orchestrator/src/compiled_pipeline.rs (TemplateCrystallizer) | VERIFY_NEEDED | plan cites line 292/335; not re-read this sweep (claim is internal wiring, not external behavior). context-compiler verifies at EXECUTING.
crates/orchestrator/src/mini_critic.rs + structural_retry.rs (BestOfN verdict reuse) | VERIFY_NEEDED | plan cites mini_critic.rs:240-270 + structural_retry.rs:277; internal-wiring claim, deferred to context-compiler.

CLAIM CLASSIFICATION
Voyager "critic-gated acquisition 3.3x" | GROUNDED (as NON-TARGET) | 05-voyager.md:122 cite [1]: "3.3x more unique items" Minecraft, vendor-reported, frozen 2024-04; F-6 plan line 53-54 + 364 already states NO reuse-rate claim. METHODOLOGY-ONLY borrow.
Voyager 3.3x transfers to ELAI coding skill-reuse | UNGROUNDED (correctly flagged in plan) | synthesis §1.5 line 104-106: "non-transferable... zero evidence for coding-task skill-reuse"; plan honors this. Flag = NON-TARGET, must never be an acceptance number.
Voyager critic-gated acquisition LOOP exists/is buildable | GROUNDED | 05-voyager.md:66-81 + 147 (CriticAgent generate->execute->critic(success,critique)->retry<=4->commit, shipped working loop). Borrowed pattern is real.
SHARD T1-T4 trust ladder (shipped real code) | GROUNDED | 00-COMPARISON:55,83,98 (T1-T4 IntEnum in skills.py, fetched 2026-06-01); plan maps to DISTINCT enum S1-S4 per §7.
Trust promotion by invocation-count + failure-rate + min-days | GROUNDED | 00-COMPARISON:92 "T4 tier never exercised (30-day gate)" confirms a min-age gate; ladder shipped in skills.py; plan Task 1 names TIER_MIN_INVOCATIONS / TIER_MAX_FAILURE_RATE / TIER_MIN_AGE_SECS constants (ELAI sets values; SHARD confirms mechanism shape).
Top trust tier human-gated (S4) | GROUNDED | 00-COMPARISON:92 + plan Task 1: S4 promote only with human_promotion_granted; SHARD's T4 30-day gate is the human/age-gated apex. Mechanism grounded; exact thresholds are ELAI design choices, NOT borrowed numbers.
SHARD-specific T1-T4 promotion threshold VALUES | UNGROUNDED-as-target (correctly internal) | SHARD threshold integers not source-verified in the dossiers; F-6 sets its OWN named constants -- these are design choices, NOT external claims. Must NOT be presented as SHARD-derived numbers.
"70% reuse" measurable | UNGROUNDED (correctly DEFERRED in plan) | NOT in VIBESEC-PAPERS (grep: no match); it is an ELAI-INTERNAL aspiration, deferred per synthesis §4 C6 line 387 + plan ACCEPTANCE line 364-366 until concept_class SSB tagging + >=20 acquisitions. NON-TARGET this session.
SHARD staleness = (now-last_verified)/expected_lifetime, flag>1.0 quarantine>2.0 | GROUNDED (shape only) | 05-voyager.md:152 + 00-COMPARISON:82; plan C6c reuses the time-decay SHAPE with ELAI's own SKILL_EXPECTED_LIFETIME_SECS / SKILL_STALE_THRESHOLD constants (its own thresholds, not SHARD's).
SCHEMA_VERSION = baseline + 1; Pattern-2 single-variant + 3-arm Kind | VERIFY (internal) | plan grounds against E-43 ssb.rs:1888 + RouteDecisionEvent ssb.rs:23884; context-compiler confirms current baseline at EXECUTING (CLAUDE.md says SCHEMA_VERSION 191 / TV 464 today; "baseline+1" must reconcile to live value).

NUMBERS THAT MUST NOT BECOME TARGETS (the core deliverable of this sweep)
- 3.3x (Voyager "more unique items") -- Minecraft tech-tree, vendor-reported, frozen 2024-04, single-digit seeds. NON-TARGET. Methodology-only. Any ELAI acceptance criterion citing 3.3x = REGRESSION to a refuted claim (synthesis §1.5).
- 70% reuse -- ELAI-internal aspiration, NO baseline exists, DEFERRED until >=20 acquisitions + concept_class SSB tagging. NON-TARGET this session. Plan's deferral checkbox (ACCEPTANCE line 364-366) records the deferral; it must stay a record, never an assertion.
- SHARD T1-T4 raw threshold integers (e.g. "30-day", "5/session") -- SHARD's own constants, NOT source-verified as portable, and NOT what ELAI ships. F-6's TIER_MIN_* constants are independent design choices. Do NOT label F-6's thresholds as "SHARD-derived".
- Any tech-tree multiplier (wood 15.3x, stone 8.5x, iron 6.4x, diamond 1-of-3) -- same provenance as 3.3x; NON-TARGET, irrelevant to coding skill reuse.
- Springdrift +10.7% P@4 / 7,056-pair (00-COMPARISON:75) -- adjacent C7 number, synthetic/unreproduced (synthesis §1.2); NOT an F-6 claim, listed here so it is not cross-imported into F-6's measurable.

ACTIONS REQUIRED
- NON-TARGET ENFORCEMENT: confirm no F-6 ACCEPTANCE checkbox or DETERMINISTIC_CHECK asserts 3.3x or 70% as a pass condition. (Verified clear: plan ACCEPTANCE line 364-366 explicitly records the deferral and asserts NO number. No action needed beyond keeping it so.)
- VERIFY internal-wiring claims at EXECUTING (context-compiler, not paper-ingestor): compiled_pipeline.rs:292/335 (check_crystallization shape), mini_critic.rs (MiniCriticVerdict Pass/Warn/Block), structural_retry.rs:277 (run_best_of_n emits mini_critic_verdict SSB row), autonomous_gate.rs:178, memory_trust_contract.rs (TrustTier wire tags) -- all CONFIRMED-in-elai.map per plan, re-grep at edit time per impact-check.
- VERIFY live SCHEMA baseline before "baseline+1": CLAUDE.md states SCHEMA_VERSION 191 today; plan says "baseline+1". Resolve the concrete integer at EXECUTING (grep SCHEMA_VERSION in ssb.rs) and reconcile if another F-N bumps in the same window.
- NO INGEST needed: Voyager (arXiv:2305.16291) and SHARD are already fully dossiered in workflow/research/shard-prior-art/; the synthesis is adversarially verified (wf_b959963a-db6). No paper-ingestor spawn required for F-6.
- DOCUMENT in F-6-decisions.md (plan Task 12 already specifies (c)): Voyager 3.3x non-transferable, NO reuse-rate claim, measurable deferred until >=20 acquisitions. This sweep confirms that decision-doc clause is mandatory and correctly scoped.

VALIDATION WARNINGS
- F-6 cites synthesis/research dossiers, NOT docs/specs/guide/steps/phase-*.txt -- this is correct for an F-plan (F-plans trace to the VibeSec/SHARD synthesis, not the original phase spec). Not a traceability defect.
- The three named claims are all GROUNDED as MECHANISMS but their NUMBERS are NON-TARGET; the risk profile is inverted (guard against numbers becoming gates, not against acting on unverified claims). Plan already handles all three correctly -- this sweep is a confirmation pass, not a blocker.
- Internal-wiring symbols marked VERIFY_NEEDED are deferred to context-compiler at EXECUTING per main-llm-role (spec-grounder does not re-read crate internals for non-external claims).
- all external-behavior sources validated (Voyager dossier, SHARD comparison, synthesis all read this session).
