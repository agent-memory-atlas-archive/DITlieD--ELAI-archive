steal analysis — odysseus (PewDiePie) Cookbook + onboarding + adjacent mechanics
paper-to-plan phase 4 ledger, in-band, 2026-06-21. source: docs/.source/odysseus-main +
odysseus-INDEX.md (cloned + indexed 2026-06-04). ledger checked against ELAI-FEATURE-MAP + J-9.

OUTCOME: 0 new plan files. The Cookbook the user asked about is ALREADY planned (J-9). 1 EXTENSION
candidate surfaced (teacher-escalation skill-learn loop) — flagged, NOT authored (needs a C-49/E-31
verification pass first to avoid a GD-1 re-statement). The rest is already-covered or a deliberate
decline. This is the steal pass doing its job: preventing a redundant plan file.

PER-MECHANIC VERDICTS (odysseus symbol -> ELAI status)

1. hwfit/fit.py (the "Cookbook": rank_models:521 / analyze_model:299 / _fit_score:198 /
   _quality_score:143 / _estimate_speed:73 / _quant_bits:243) -> ALREADY-COVERED (J-9).
   J-9 already cites this exact file as HW-12 ("structural prior art, read-only") and Task 3 already
   steals "the odysseus STRUCTURE (fit x footprint headroom) with ELAI-derived weights documented in
   the decisions doc (HW-12, GD-5)". J-9's HardwareModelCatalog IS ELAI's Cookbook. Re-proposing it
   would be a GD-1 P0. NO new file.

2. teacher_escalation.py (is_self_hosted:49 / evaluate_turn_regex:91 trigger / escalate_and_learn:368
   / maybe_escalate:428 / run_teacher_inline:473) -> EXTENSION CANDIDATE (the one genuine net-new).
   Mechanic: a self-hosted model, on a DETERMINISTIC local-struggle trigger (regex over the turn),
   escalates to a cloud "teacher" AND distils a reusable skill from the teacher's solution so the
   local lane improves over time. ELAI overlaps this heavily — E-31 tier matrix (the escalation
   routing), C-49 experience distillation [DARK] (iterative prompt-policy distillation, persistent
   corpus deferred), I-30 verifier-ranked SFT exhaust, D-5 bandit. The NET-NEW is the COUPLING: a
   deterministic struggle-DETECTOR wired to an inline teacher call whose output is a verifiable,
   eval-gateable skill artifact. This is aligned with the "verifiable self-hosted dev environment"
   wedge (the teacher turn is a trust-gradeable artifact; the learned skill can be gated by the J-56
   eval corpus before promotion, TD-3). NOT AUTHORED THIS PASS: authoring it correctly requires
   reading the C-49 + E-31 + I-30 detail files to scope the EXTENSION without re-stating them (GD-1).
   Flagged for a follow-on unit if the user wants it.

3. tool_index.py (ToolIndex RAG tool selection, retrieve:260 vector top-k, get_tools_for_query:431)
   -> ALREADY-COVERED. J-12 H5 (BM25 top-1 skill retrieval over F-6 trust tiers) + J-24 context-
   frugal tools + the harness Tool Search mechanism. Same idea (select tools by retrieval, not dump
   all schemas).

4. memory.py + memory_vector.py + memory_extractor.py + skill_extractor.py (two-tier flat + chroma +
   LLM extraction) -> ALREADY-COVERED. ELAI crates/memory: E-5 (Mem0 ADD/UPDATE/DELETE/NOOP + Zep
   bi-temporal), H-31/H-32 (preference + code-grounded fact memory), H-33 (governance), J-21 (semantic
   validation + poisoning). ELAI's substrate is broader (bi-temporal, poisoning defense, citation
   validation). No steal.

5. deep_research.py (IterResearch DeepResearcher, _create_plan / _generate_queries / _should_stop)
   -> ALREADY-COVERED. B-40 FlashResearch tree + the deep-research skill + J-40 research modes /
   extractive compression. ELAI's is the same IterResearch lineage.

6. prompt_security.py (untrusted_context_message — wrap untrusted content with a labeled boundary)
   -> ALREADY-COVERED. G-1 taint propagation (Provenanced<String> + capability_gate) + the untrusted-
   content boundary pattern. ELAI's is stronger (dataflow taint, not just a wrapper).

7. endpoint_resolver.py (resolve_endpoint:205, tailscale host resolution:77, build_chat_url/headers)
   -> ALREADY-COVERED by J-30 BYOK custom endpoints + B-37 multi-runtime backend abstraction. MINOR
   RIDER recorded: the tailscale-host resolution leg (resolve_tailscale_host:77) is a small
   self-hosted-friendly convenience J-30 could note (reach a sidecar by tailscale name); rider on
   J-30, not a unit.

8. context_compactor.py + context_budget.py (DEFAULT_HEADROOM=0.85) -> ALREADY-COVERED. phase-3E
   context budget allocator + E-25 compaction ladder + H-50 regime-aware policy. CONVERGENCE NOTE:
   odysseus' DEFAULT_HEADROOM=0.85 is the SAME folklore constant as ELAI's CLOUD_TRIM_FRACTION=0.85
   (context_policy.rs:70) — and J-34 (empirical context calibration) already exists to replace that
   hardcode with a measured knee. Two independent codebases landing on 0.85 by feel is itself
   evidence for why J-34's measure-it approach matters. No steal; reinforces J-34.

9. task_scheduler.py (cron-like ScheduledTask background execution) -> DECLINE (assistant-direction).
   This is the personal-assistant "run my tasks on a schedule" feature, not the coding-trust wedge.
   Per the direction verdict (elai-vs-odysseus-machinaos-2026-06-21.md), do not chase the assistant
   category. Revisit ONLY if ELAI adds scheduled autonomous WORK runs (and even then the harness
   already has schedule/cron tooling). Recorded as a deliberate decline.

10. event_bus.py -> ALREADY-COVERED (SSB broadcast-after-commit + task_runner event dispatch). Trivial.

11. onboarding / packaging (Docker up + the Cookbook recommending a model for your box) -> NOT A PLAN
    UNIT, but the most actionable non-code lesson: the 30K-stars-in-2-days reason is "docker compose
    up and it runs, then it tells you which model fits." ELAI's setup is heavier (sidecar boot, tunnel,
    env vars). Recorded as a packaging/onboarding recommendation for whoever owns ELAI's first-run UX
    (the FirstRunWizard is already noted absent in B-11/B-12 PARTIAL) — not a steal, a devops priority.

LEDGER SUMMARY: 0 NEW. 1 EXTENSION-candidate (teacher-escalation skill-learn, flagged, unauthored
pending C-49/E-31/I-30 verification). 1 minor rider (tailscale endpoint -> J-30). 1 reinforcement
(0.85 folklore -> J-34). 1 decline (task_scheduler, assistant-direction). 5 already-covered
(tool-RAG, memory, deep-research, prompt-security, context-compaction). 1 onboarding/packaging
recommendation (not a unit). Cookbook = J-9 (already planned).

DIRECTION TIE-IN: every "already-covered" above is more grounded/broader in ELAI than in odysseus
(taint vs wrapper, bi-temporal poisoning-aware memory vs flat+chroma, measured calibration vs 0.85
folklore). That asymmetry IS the "verifiable member of the self-hosted category" wedge made concrete:
ELAI is not behind odysseus on these mechanics, it is ahead on the trust dimension. See
elai-vs-odysseus-machinaos-2026-06-21.md for the direction verdict.
