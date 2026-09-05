WIRING FIX -- PROPAGATION MANIFEST
Written 2026-06-02. Companion to WIRING-FIX-CLAUDE-CODE-WORKFLOW.md and WIRING-FIX-ELAI.md.
Records exactly which harness files were changed/created in F:\.ELAI_workflow so the fix can be
propagated to the sibling workflow harnesses. Targets:
  D:\.apps_workflow      (not a git repo; has settings.json; planning-workflow present; no b-plan-runner)
  D:\.liteGAME_workflow  (not a git repo; has settings.json; planning-workflow present; no b-plan-runner)
  D:\.REGRET_workflow    (git repo; no settings.json, only settings.local.json; planning-workflow present)
Survey ground truth: none of the three has wiring_contract_check.py, wiring-contract-validator.md, or
b-plan-runner today; all three have .claude/rules + planning-workflow + the 4 core rules + commit-cadence.

=========================================================================================
TIER A -- DROP-IN PORTABLE (copy verbatim; the doctrine is the "never repeat the mistake" core)
=========================================================================================
These are project-agnostic. Copy as-is into each target's identical path.

  .claude/rules/wiring-enforcement.md
      The fail-closed doctrine (R1-R9) + walking-skeleton PRE_GATE + freeze-the-gate + no-default-off.
      This is the single highest-value propagation artifact. It is what governs the LLM's planning and
      gating behaviour. Contains zero ELAI-specific paths. Copy verbatim.

  .claude/tools/gate_freeze_check.py
      Git-independent, stdlib-only (blake3 if importable, else sha3_256). Resolves its own root from
      __file__, reads .claude/gate-manifest.txt relative to .claude/. Copy verbatim. No edits.

=========================================================================================
TIER B -- COPY THEN EDIT ONE LIST/LINE (small per-project adaptation)
=========================================================================================
  .claude/rules/conditional-routing.md
      ADD the single dispatch line (do not overwrite the file -- the targets have their own routing):
      "Shipping a feature / writing a done-gate or wiring check / editing a WIRING_CONTRACT / planning a
       phase-step-babystep / auditing a feature for \"done\" / touching any feature flag: read
       .claude/rules/wiring-enforcement.md. Every rule (R1-R9) mandatory; fail-closed by default."

  .claude/skills/planning-workflow/SKILL.md
      ADD the two injected blocks (the targets all have planning-workflow):
        - in the PRE-GATE INJECTION step: the WALKING-SKELETON PRE_GATE paragraph (vertical-slice refusal).
        - in the ACCEPTANCE CRITERIA step: the OFF-AGENT PROOF paragraph.
      Both reference .claude/rules/wiring-enforcement.md and are project-agnostic. Splice them into the
      target's equivalent steps; do not blind-overwrite (the targets' SKILL.md may differ in wording).

  .claude/gate-manifest.txt
      Copy the header, then list the GATE files that actually exist in that target. For a target with only
      Tier A files, the manifest is:
        .claude/rules/wiring-enforcement.md
        .claude/tools/gate_freeze_check.py
      Add the target's own done-gate (e.g. test_determinism_gate.py -- all three targets have one) and any
      project gate-owned data files once they exist. Then run once, out-of-band:
        python .claude/tools/gate_freeze_check.py --lock

=========================================================================================
TIER C -- ELAI-TUNED (do NOT blind-copy; these assume ELAI's evidence store)
=========================================================================================
  .claude/tools/wiring_contract_check.py
      Logic is portable but it is wired to ELAI's evidence: elai.map (SCIP graph) and the SSB sqlite db.
      Paths are now env-overridable (ELAI_MAP, ELAI_PROJECT_ROOT, ELAI_SSB_DB). The targets do NOT have
      elai.map or an SSB db; their off-agent artifact is different (they ship test_determinism_gate.py).
      To reuse: either (a) point the env vars at the target's own map/event-db, or (b) keep the script
      ELAI-only and let each target's existing determinism gate be its Layer-3/4 proof, governed by the
      Tier-A doctrine. Recommended: (b) for now; do not fabricate an elai.map for a non-ELAI project.

  .claude/agents/wiring-contract-validator.md
      Thin runner over wiring_contract_check.py. Propagate only if the target also adopts the script (Tier C
      option a). Otherwise the target's auditor enforces the Tier-A doctrine against its own determinism gate.

  .elai_cc/smoke-manifest.toml , .elai_cc/wire-dark-allowlist.toml
      Gate-owned DATA, ELAI-specific (action_types, ELAI_*_ENABLE flags). Do not copy the contents. Each
      target creates its own equivalents if/when it adopts a runtime-evidence gate, using the same schema
      (documented in the file headers).

=========================================================================================
ELAI RUNTIME (separate, NOT a harness propagation) -- WIRING-FIX-ELAI.md
=========================================================================================
The Rust runtime redesign (EntrypointSetLoader empty=FAIL, DkgReachabilityAuditor producer-without-caller
=FAIL, ObservableEffectRunner SSB-row-or-FAIL, AutoAdvanceEvaluator requires Layer 3+4, gate fileset into
the Phase 4H LOCKED set) is ELAI-internal and C/F-plan-sized. It is grounded design, not yet built; it does
NOT propagate to the sibling workflows. Stage it as its own plan (orchestrate / b-plan).

=========================================================================================
PROPAGATION ORDER (per target)
=========================================================================================
  1. Copy Tier A (wiring-enforcement.md, gate_freeze_check.py).
  2. Splice Tier B (conditional-routing line, planning-workflow two blocks).
  3. Write gate-manifest.txt for that target; run gate_freeze_check.py --lock once.
  4. (Optional) Tier C if the target adopts the runtime-evidence script.
  5. Verify the doctrine loads: confirm the conditional-routing dispatch line resolves.
=========================================================================================
PROPAGATION STATUS -- DONE 2026-06-02 (all 3 targets)
=========================================================================================
Shipped to D:\.apps_workflow, D:\.liteGAME_workflow, D:\.REGRET_workflow:
  - Tier A: wiring-enforcement.md + gate_freeze_check.py copied verbatim. CONFIRMED present.
  - Tier B (router): the wiring-enforcement dispatch line added to each conditional-routing.md
    (anchored after "Architecture choice"; flag wording generalized off ELAI_*_ENABLE).
  - gate-manifest.txt written per target (the 2 portable files) and BASELINE LOCKED; verify = OK exit 0
    in all three (the copied freeze tool self-resolves root + manifest correctly).
INTENTIONALLY SKIPPED on targets (documented, not an omission):
  - The inline planning-workflow PRE_GATE/ACCEPTANCE splice. The three targets' planning-workflow
    skills diverge (apps = baby-step "Strengthen BLOCK 2" style; liteGAME = ELAI numbered-step style;
    REGRET = no such improvement section). The router dispatch line fires on "planning a phase-step-
    babystep", and wiring-enforcement.md self-contains the walking-skeleton PRE_GATE + off-agent-proof,
    so the doctrine governs planning in all three WITHOUT mangling divergent files. Splice inline later
    per-target only if a target wants it in its improvement checklist too.
  - Tier C (wiring_contract_check.py, validator agent, smoke-manifest, allowlist): not copied. The
    targets have no elai.map / SSB db; do not fabricate one. Their existing determinism gate is the
    runtime-evidence leg, governed by the Tier-A doctrine.
TODO per target (optional hardening): add the project's own done-gate (determinism/test gate) to its
  gate-manifest.txt and re-lock; `pip install blake3` to upgrade the freeze hash from sha3_256.
End of WIRING-FIX-PROPAGATION.md.
