# F-Plan Index — VibeSec/SHARD steal-and-build, wiring-enforced (10 sessions)
Date: 2026-06-01
Source synthesis: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md (the 10 grounded
  proposals + their adversarial verdicts, verification run wf_b959963a-db6).
Grounding files: workflow/reasoning/VIBESEC-RESEARCH-2026-06-01-LANDSCAPE.md, VIBESEC-TOOLS-MATRIX,
  VIBESEC-PAPERS, workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md.
Predecessors: B-plan (B-1..B-42), C-plan (C-1..C-59), D-plan (D-1..D-30), E-plan (E-1..E-55).

Predecessor reading (mandatory before any F-N execution):
  workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md (the why + the per-item scope)
  docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md (current state)
  docs/thefinalglance/E-plan/E-30_public_agent_incidents.md (THE format template every F-N mirrors)
  workflow/sessions/s-night-2026-05-30/handoff/HANDOFF-NIGHT-2026-05-30.md (the failure this plan exists to prevent)
  This file (F-0_INDEX.md), then F-SUMMARY_INDEX.md.

## Why F-plan exists (read this before anything else)

The night-run of 2026-05-30 cross-checked 187 plans and found the spread: ~52% SHIPPED-WIRED, ~15%
SHIPPED-WIRE-DARK (code lives, ZERO production callers), ~17% SHIPPED-ENV-GATED (behind
ELAI_*_ENABLE=1 default-OFF), ~10% PARTIAL, ~5% NOT-SHIPPED. WIRE-DARK + ENV-GATED + PARTIAL is
~40-45% of the inventory. The B/C/D/E plans ALREADY carried a WIRING_CONTRACT block - and it did not
save them, because the contract was a DOCUMENTED block, never an ENFORCED, EXECUTED done-gate. Even
the wires landed during that night-run shipped default-off (E-17 contextual_retrieval, B-40
frame_idea, D-16 UniCR, E-49) - dark by another name. The handoff's own words: "the production graph
is sparse - many modules ship symbols + tests, but the canonical orchestrator/cli/dashboard entry
points never invoke them" + "~232 invisible backend signals (producer exists, no consumer)".

F-plan's single job: make wiring AND integration a HARD, EXECUTED, NON-NEGOTIABLE done-gate so the
features below cannot be marked complete while wire-dark. The F-plan content is the 10 VibeSec/SHARD
mechanisms from the synthesis doc; the F-plan FORM is the wiring doctrine below.

## Status legend
- PLANNED — file written, work not started (all 10 F-N at generation)
- IN PROGRESS — active session
- DONE — passes the FULL done-gate in the doctrine (cargo green is necessary but NOT sufficient)
- DEFERRED — RunPod / external dependency gated (named per F-N)

================================================================================
THE F-PLAN WIRING DOCTRINE (HARD RULES W-1..W-10 — every F-N inherits these)
================================================================================

These are the enforcement clauses that the B/C/D/E plans lacked. Each F-N's WIRING_CONTRACT block is
written to satisfy them, and each F-N's ACCEPTANCE makes them executable done-gates. The existing
harness tooling is the enforcement engine - F-plan does NOT build new tooling, it MANDATES the
existing tools as gates.

W-1  NO-WIRE-DARK DONE-GATE (the core rule). A session is DONE only when
       python .claude/tools/wiring_contract_check.py --bn-file <F-N>.md --run-trigger
     returns PASS (exit 0): every produces_symbol exists as a DEF/SCIP_DEF in elai.map, every
     consumes_symbol has a real CALL/REF, every reachable_from anchor is reached by BFS over
     CALL_RESOLVED edges within depth 8, stub_ban finds zero forbidden literals, AND the
     observable_effect trigger actually RAN and emitted the expected SSB row + artifact. `cargo check
     --workspace` GREEN is necessary but NOT sufficient. The wiring-contract-validator agent
     (.claude/agents/wiring-contract-validator.md) runs this; the FSM may NOT transition to COMPLETE
     without its PASS. This is the clause the B/C/D/E plans never enforced.

W-2  NO DEFAULT-OFF ENV GATE AS THE TERMINAL STATE. A feature MAY carry a kill-switch env var to
     DISABLE it, but its DEFAULT must be ON / wired into the production path. If staged rollout needs
     default-off first (e.g. an A/B recall measurement), the SAME F-N MUST include the flip-to-
     default-on + smoke step, and the session is NOT DONE until default is ON and the observable_effect
     runs with NO env override set. Every F-N's stub_ban carries a clause forbidding its flag's
     default-off terminal state. (This is the exact escape hatch the night-run wires fell through.)

W-3  PRODUCTION ENTRYPOINT REQUIRED. Every produced capability must be reachable_from at least one
     REAL production entrypoint: a CLI subcommand / slash-command (crates/cli), an FSM transition or
     evidence gate (crates/orchestrator/src/fsm.rs or task_runner.rs), a PreToolUse/PostToolUse hook
     (crates/plugins or hook_dispatcher.rs), or a Tauri command (crates/dashboard-v2/src-tauri).
     A symbol with only #[cfg(test)] callers is wire-dark = NOT DONE.

W-4  FRONTEND PAIR MANDATORY FOR ANY DASHBOARD-SURFACEABLE SIGNAL. If the F-N emits an SSB action_type
     or any signal a human should SEE, it MUST name a frontend_pair (the Svelte consumer + the Tauri
     command) and python .claude/tools/cross_lang_diff.py MUST pass (no Rust<->TS drift). A backend-only
     signal with no UI consumer is NOT DONE (~232 such invisible signals existed at the night-run).
     The only exception is a pure internal enforcement gate with no user-facing meaning, justified
     explicitly in the F-N's WIRING_CONTRACT.

W-5  OBSERVABLE_EFFECT IS A LIVE END-TO-END TRIGGER, NOT A UNIT TEST. observable_effect.trigger must be
     a real command a human can run (a CLI invocation or a dashboard action) that produces
     observable_effect.expect_ssb (a row in the live ssb.db) AND observable_effect.expect_artifact (a
     file/stdout a human can inspect). "cargo test passes" is NEVER the observable effect. Where a live
     model run is required and unavailable (RunPod-pending), the F-N must ship a DETERMINISTIC FIXTURE
     HARNESS that exercises the SAME production code path (not a mock of it) and label the live-run
     acceptance as a SEPARATE, explicitly-gated checkbox.

W-6  INTEGRATION CHECKPOINT THROUGH THE DASHBOARD. Each F-N's ACCEPTANCE must include >=1 item verified
     THROUGH the dashboard (the standing user demand: "make sure everything works THROUGH the
     dashboard, not just unwired backend"). The dashboard-handoff skill + dashboard-smoker agent verify
     it. (Trivially-headless F-Ns with no UI surface declare this N/A with a one-line justification.)

W-7  NO HALLUCINATED INTEGRATION POINTS. Every EXISTING symbol the new code attaches to
     (consumes_symbols, reachable_from anchors, every MODIFY target in Files-touched) must be confirmed
     real in elai.map AT AUTHORING TIME. A contract that hooks into a non-existent function is the root
     cause of "I had to wire stuff later". This is the B-40/B-42 bad-cite failure mode applied to
     wiring. Author greps elai.map; the validator re-greps.

W-8  PRODUCES/CONSUMES SYMMETRY. Every produces_symbol meant to be invoked must appear in a
     consumes_symbol or a reachable_from chain WITHIN THE SAME F-N. A produced function with no in-plan
     caller is wire-dark by construction - the plan must name its caller, not defer it to "later".

W-9  SSB VARIANT + GOLDEN LOCK. New SSB action_types flow through the action_type golden file (E-45
     pattern); the SCHEMA_VERSION bump is named in the WIRING_CONTRACT; cross_lang_diff +
     action_type_lock must be green. Reuse the Pattern-2 single-variant-with-Kind-discriminator
     collapse where the night-run / SCHEMA discipline applies (baseline SCHEMA_VERSION 191 per
     CLAUDE.md; never hard-code a final number - say "baseline + N").

W-10 TIERED-GATE RULE (the synthesis cross-cutting insight). Any axis needing NETWORK or SUBPROCESS at
     gate time must be SPLIT: the in-process Tier-1 ships genuinely fail-closed and IS the done-gate;
     the network/subprocess/RunPod Tier-2 is a separately-gated sub-phase that MUST document which
     guarantee it delivers and MUST NOT advertise fail-closed if it is fail-open (a fail-open gate that
     implies safety it does not deliver is worse than no gate).

DONE DEFINITION (explicit, the FSM may not reach COMPLETE without all of these):
  (1) cargo check --workspace GREEN, (2) auditor PASS (sole test runner), (3)
  wiring_contract_check.py --run-trigger PASS, (4) cross_lang_diff PASS if frontend_pair present,
  (5) the W-6 dashboard E2E item verified, (6) memory-writer ran. cargo-green-alone = NOT DONE.

================================================================================
THE 10 SESSIONS (1:1 with the synthesis clusters C1..C10, in REVISED scope)
================================================================================

  F-N   title                                          steal/new  novelty           tier
  F-1   VibeGate unified fail-closed write-gate          HYBRID    GENUINELY_NEW     1 (Tier-1 axes now)
  F-2   code-vuln ruleset + first measured insecure-rate HYBRID    GENUINELY_NEW     1 ruleset / 2 number
  F-3   CaMeL/Tool-Filter data-plane defense             NEW       GENUINELY_NEW     1 ToolFilter / 3 CaMeL
  F-4   test-integrity "cant-lie" gate                   HYBRID    PARTIALLY_COVERED 1
  F-5   cross-subsystem staleness-quarantine cascade     HYBRID    GENUINELY_NEW     3 (after B-30)
  F-6   learned-skill library (trust ladder + critic)    HYBRID    PARTIALLY_COVERED 3
  F-7   safety-lattice proof + CBR template ranking      HYBRID    GENUINELY_NEW     3 (split)
  F-8   SCF symbolic conflict (self-mod first)           HYBRID    GENUINELY_NEW     3 (self-mod only)
  F-9   hallucinated-dep write-time gate                 HYBRID    PARTIALLY_COVERED 2 (after D-11)
  F-10  security-monotonicity iteration guard            NEW       GENUINELY_NEW     2 (classifier first)

Cluster->F-N map is identity (C1->F-1 ... C10->F-10). Each F-N's "Existing state" block names the B/C/D/E
plans it composes with (the synthesis doc records these per item; the authored F-N re-greps to confirm).

================================================================================
SCHEMA_VERSION BUMP SITES (baseline 191 per CLAUDE.md; each F-N names its own scope)
================================================================================
  F-1   +vibesec_code_vuln_blocked, vibesec_secret_blocked (Tier-1); vibesec_dep_hallucinated_blocked,
        vibesec_skill_blocked reserved for Tier-2 (Pattern-2 VibeGateBlocked + 4-arm Kind)
  F-2   +security_rule_blocked, insecure_rate_measured
  F-3   +tool_filter_committed, untrusted_var_quarantined (capability_flow_blocked reserved Tier-2)
  F-4   +test_integrity_verified, anti_fabrication_block (+ stdout_hash field on TestOutcomePayload)
  F-5   +staleness_quarantine_broadcast
  F-6   +skill_acquired, skill_trust_promoted, skill_demoted_stale
  F-7   +safety_lattice_verified (E-53 amendment), template_utility_ranked
  F-8   +symbolic_conflict_detected
  F-9   +write_time_slopsquat_blocked
  F-10  +security_class_regression_blocked

================================================================================
SEQUENCING (from synthesis section 5 — leverage x feasibility-now x defensibility)
================================================================================
  TIER 1 (do first; in-process, genuinely fail-closed, on-thesis):
    F-2a (vuln-class ruleset) + F-1 Tier-1 (code-vuln + secret) co-ship; then F-4 (cant-lie); then
    F-3 phase 1 (Tool Filter). These need no network, no RunPod, no subprocess at gate time.
  TIER 2 (the measured number + the slop gate; RunPod / D-11 gated):
    F-2b (Qwen secure-rate number - THE STRATEGIC KEYSTONE: R1 + R2 + first-in-category), F-9
    (write-time slop gate, after D-11 ships), F-10 (security-monotonicity, classifier-build first).
  TIER 3 (governance + composition; sequencing-dependent learning loops):
    F-6 (skill library a/b/c), F-5 (staleness cascade, after B-30 + concept_stale retrieval audit),
    F-8 (SCF self-mod path), F-7 (E-53 amendment + standalone CBR plan), F-3 phase 2 (full CaMeL,
    gated on token-overhead measurement).

THE ONE HEADLINE MOVE: F-1 Tier-1 + F-2a -> F-2b. Convert ELAI's rules-as-data + PreToolUse +
SSB-journal substrate into the one thing the VibeSec market sells but nobody measures (a measured
pre-write before/after insecure-rate). It is simultaneously ELAI-DIRECTION R1 (a real number on its
own engine), R2 (the anti-Replit distribution demo), and a first-in-category claim - and it is
defensible BECAUSE of the SSB journal + ELAI_SEED determinism ("a security control you can audit and
reproduce").

================================================================================
HARD RULES INHERITED FROM CLAUDE.md (apply to every F-N)
================================================================================
- blake3 only (never blake2/sha2 for non-RFC uses).
- No .unwrap()/.expect() in production paths (test cfg + main entrypoint allowed).
- model.generate() + LogitsProcessor only; FA2 mandatory on the 7B.
- Anti-slopsquatting: verify package name + version + last-published date BEFORE adding any dep.
- No new external deps without user confirmation (flag per F-N: F-3 PromptArmor-class, F-9 Socket
  Firewall wrap (ops-only), F-10 CWE-classifier ruleset, F-6 any embedding/skill-store dep).
- Tier C isolation mandatory for any CLI subprocess.
- One change at a time; the auditor is the sole test runner; the main LLM never runs tests.
- Grounded-by-default: every external-behaviour claim in an F-N cites the synthesis doc + its primary
  source; UNVERIFIED claims are labelled (the synthesis doc section 1 records the corrections already
  forced - do NOT re-introduce CaMeL 2.82x as primary, Springdrift +10.7% as a target, etc).

================================================================================
SESSION LOG (populated as F-Ns ship)
================================================================================
(Empty at generation. Each shipped F-N adds a row: context-compiler -> integration-explorer ->
spec-grounder -> executor -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator (RUN_TRIGGER
PASS REQUIRED) -> dashboard-smoker (if frontend_pair) -> memory-writer -> COMMITTING.)

================================================================================
POST-F-10 EXTENSIONS (added 2026-06-02 — PLANNED, implementation pending)
================================================================================
Four parts added after the original C1..C10 set, each grounded in live code + primary sources and
adversarially verified (research run wm3rajyyo; drafts + decision basis at
workflow/research/F-plan-next/00-RECOMMENDATIONS.md). All PLANNED; another session orchestrates them.

  F-2c  low-FP CWE rules (798/321/532/95) on the already-registered Py+TS grammars via the existing
        Composite engine, then RUN the shipped `elai bench seccodeplt` for the first measured
        static-gate baseline. EXTEND of F-2. No new dep, no ast_adapter edit, no SSB variant.
        (F-2c_low_fp_cwe_rules_seccodeplt_baseline.md)
  F-11  close ONE config-only DGM generation under the existing guards. The DGM substrate ships
        INERT (config_archive sigmoid/parent-select + bandit + two-gate + trap-tasks all built and
        tested, ZERO production callers — verified 3-file grep); only the loop driver is missing.
        Mutant = config variant only; explicitly wires the is_security_sensitive filter the archive
        admit path bypasses. Wire-dark behind feature darwinian-evolve + ELAI_DARWINIAN_LOOP_ENABLE
        (deliberate W-2 deviation, justified in-file). +darwinian_admit SSB.
        (F-11_close_one_config_generation.md)
  F-12  semantic contradiction pre-filter: a LOCAL deterministic-argmax NLI judge (NOT the cloud
        lane — cloud LLM breaks ELAI_SEED per arXiv:2506.09501) between audit_conflicts Surface and
        user adjudication; reuses MemoryConflictDetected with conflict_kind="semantic_contradiction"
        (NO SCHEMA bump); also supplies the first production caller of the currently wire-dark
        audit_conflicts. Tier-2 NLI ONNX model is ops-gated; trait seam ships the logic dark of it.
        (F-12_semantic_contradiction_local_nli.md)
  F-13  EXTEND_EXISTING (deliberately minimal, not the Backup-Actor mechanism): archive the stale
        ssb.db before the one destructive schema-bump DROP (ssb.rs:14342) so audit history is not
        silently destroyed. No periodic committer, no git dep on agent state. Open W-2 disposition
        recorded in-file. +session_archived SSB. (F-13_session_archive_schema_bump.md)

SCHEMA bump sites: F-11 +darwinian_admit; F-13 +session_archived; F-12 reuses MemoryConflictDetected
(free-form conflict_kind, no bump); F-2c adds no SSB variant.

================================================================================
NEXT STEP
================================================================================
Start with the TIER-1 headline move: F-2a + F-1 (Tier-1 axes) co-shipped, then F-2b once a RunPod 2x3090
window opens. Each F-N file carries its own Pre-requisites line and the full WIRING_CONTRACT done-gate;
respect the doctrine W-1..W-10 above on every session - that is the whole point of this plan folder.

End of F-0_INDEX.md.
