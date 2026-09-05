# L-21: weak-lane verification-intensity policy — gen-time verification scales inversely with model capability (the small-model orchestrator steal)

STATUS: PLANNED (L-T4 WAVE 2; compressed form, steal track; nature EXTENSION)

- Track L-T4 (MEASURED-COMPETITOR / EVAL-SUPPLY) | lever CANT-WRECK/GROUNDED | grounding STRONG (codebuff base2.ts read 2026-07-03; ELAI anchors PLAN-asserted, re-grep at edit per W-7) | nature EXTENSION(J-51/J-55/C-32/H-7/B-41) | effort M | WAVE 2 | depends: J-51 ModelCapabilityProfile (or ships a minimal profile slice), I-51 models.toml where present

- Premise: codebuff runs DeepSeek/Kimi/GLM-class models as production coding agents by making verification intensity a function of model tier: the free (weak-lane) prompt makes a model-matched reviewer MANDATORY and fans out reviewer + typecheck + full-typecheck + tests IN PARALLEL mid-turn, while the default (strong-lane) prompt makes review skippable. ELAI routes WHO does a role (H-7), WHICH action surface a model gets (J-55), and WHETHER a model is eligible for a tool-heavy role (J-51) — but no unit scales gen-time VERIFICATION intensity by generator capability; C-32's MiniCritic/StructuralRetry loop is one fixed policy for every lane. For ELAI's own Qwen2.5-Coder-7B local lane this is the cheapest remaining lift: catch-and-fix weak-lane output IN the generation loop so what reaches the auditor is pre-corrected, without spending anything extra on strong lanes.

## Steal/build
ONE deterministic policy object, data-driven, observe-first.

VerificationIntensity: a table keyed by (ModelCapabilityProfile tier x role) resolving to {mini_critic_passes, structural_retry_budget, mid_turn_checks (scoped compile / scoped test on/off), reviewer_dispatch: mandatory|optional|off, parallel_fanout_cap}. Data lives beside the model catalog (I-51 models.toml extension or a sibling verification_intensity.toml — decide at edit; DATA never code, FS-1 spirit); resolution is a pure function (TD-3 — no LLM chooses intensity); consumed at (a) the C-32 gen-time seam (MiniCritic pass count + StructuralRetry budget become table-driven instead of constants), (b) the mid-turn check dispatch in task_runner (scoped cargo-check/test fan-out under the existing parallel cap — composes BSP spawn_parallel; NEVER full-workspace, house scoped-test rule), (c) the reviewer/audit dispatch seam (B-41 per-level modes: a weak-lane changeset marks reviewer-role dispatch mandatory pre-audit; the reviewer is generator-side ADVISORY input to the executor's retry loop and explicitly NOT the audit verdict — the Track-20 B7 decline stands, the auditor's authority is untouched).

FAIL-CLOSED DEFAULT: an unknown/unprofiled model resolves to the HIGHEST intensity tier, never baseline. INVARIANT: intensity can only ADD verification relative to today's fixed policy; no row may reduce the auditor gate, the MR floor, or any frozen checker (R8 — the table cannot weaken a mutable=0 check). Observe-first (GD-4): ships journaling intensity decisions without changing behavior until the FP/cost delta is bounded on >=3 real runs; the behavioral flip rides a kill-switch ELAI_VERIFICATION_INTENSITY_DISABLE (W-2, default-ON once promoted).

Done-number: the L-20/J-107 bench family measures the lift — weak-lane verified-outcome rate + tokens/correct with the table ON vs OFF (ELAI-measured, ID-5; reuse the K-15/I-27 runner, GD-1). The codebuff design is the rationale; no codebuff number is borrowed.

EXCLUDED: per-model hand-tuned prompt forks (ALREADY-SHIPPED I-36 compiled variants); routing changes (H-7/J-55 own those); any LLM-judged "this output needs more review" (TD-3 — the table is keyed by profile, not by output vibes).

## Existing ground truth (PLAN-asserted from ELAI-FEATURE-MAP.md 2026-07-03; re-grep at edit)
- C-32 WIRED: SkeletonGenerator BLANK_N + MiniCritic + structural retry in task_runner (map 1.12).
- J-51 PLANNED: ModelCapabilityProfile.tool_call_adherence + advisory role-gate (+ToolCallAdherenceMeasured); J-55 PLANNED ModalityRouter (the sibling policy on the same profile — L-21 is the verification axis to J-55's action axis).
- H-7/E-31 PARTIAL: calibrated role routing over PerRoleRuntime; E-31 tier matrix (docs/strategy/E-31-tier-matrix.md).
- B-41 PARTIAL: per-level auditor modes + per_level_audit_dispatch; I-51 PLANNED models.toml; BSP spawn_parallel WIRED (map 1.3).
- Track-20 B7 decline (Section 4): generator-side code-review-as-a-tool blurs the auditor boundary — L-21's reviewer leg is bounded by it (advisory-to-executor only).

## Integration
- crates/orchestrator/src/verification_intensity.rs (the table + pure resolver) + the C-32/task_runner/B-41 consumption seams; the TOML data file beside the model catalog.
- frontend_pair: an intensity badge on the runtime/route panel (RuntimePanel family; decide at edit, W-4).

## WIRING_CONTRACT
produces_symbols:
  - orchestrator verification_intensity resolver + its TOML-backed table
  - the table-driven MiniCritic/StructuralRetry budgets + mid-turn scoped-check dispatch + mandatory-reviewer marking at the three named seams
consumes_symbols:
  - C-32 mini_critic / structural_retry (the loop it parameterizes — do NOT re-implement, GD-1)
  - J-51 ModelCapabilityProfile (the key) + I-51 catalog (the data home) + B-41 per_level_audit_dispatch (the reviewer marking) + BSP spawn_parallel (the fan-out)
reachable_from:
  - the task_runner generation loop on any weak-lane-profiled backend (real production path; test-only caller never satisfies, R7)
stub_ban:
  - NO LLM-chosen intensity; NO row that weakens the auditor gate / MR floor / any frozen checker; NO full-workspace test in the mid-turn fan-out; NO silent baseline on an unknown profile (fail-closed to highest); NO .unwrap() on prod paths
frontend_pair: intensity badge (decide at edit) or justified N/A.
observable_effect:
  trigger: "run one bounded task on the local Qwen2.5-Coder-7B lane, then the same task on a frontier cloud profile (fixture-tier deterministic run now; live local arm remote-sidecar-gated, W-10)"
  expect_ssb: "prefer-reuse mini_critic / RouteDecisionEvent rows carrying the resolved intensity (else +Kind VerificationIntensitySet, net 0..+1 — re-grep LIVE baseline per W-9): weak lane journals passes>=2 + scoped-check fan-out + reviewer=mandatory; cloud lane journals baseline"
  expect_artifact: "the journaled intensity decisions recomputable from profile + table (GD-2). NEGATIVE ARMS: an unprofiled model journals HIGHEST intensity (never baseline); with ELAI_VERIFICATION_INTENSITY_DISABLE=1 behavior is byte-identical to today's fixed policy; a table row attempting to set reviewer=off on the weak lane below the floor FAILS validation at load"

## Grounding
- codebuff source (local, read 2026-07-03): agents/base2/base2.ts createBase2 mode split — free mode: FREEBUFF_REVIEWER_AGENT_ID_BY_MODEL data map, "You must spawn a <reviewer> ... in parallel with typechecking or testing", parallel reviewer+typecheck+tests fan-out; default mode: review skippable. agents/base2/base2-free-glm.ts (thin parametric wrapper). Design-rationale only; no numbers borrowed.
- ELAI anchors quoted above; ledger checked (B7 decline respected; TD-3/GD-4/R8 bind as stated).

## COMPOSES-WITH
J-51 (the profile key), J-55 (the sibling axis — resolve both from one profile read), C-32 (the parameterized loop), H-7/E-31 (routing stays theirs), B-41 (dispatch marking), I-51 (data home), L-20/K-15/I-27 (the measured done-number), maintainability-review + production-caller-gate (the untouched authorities).

End of L-21.
