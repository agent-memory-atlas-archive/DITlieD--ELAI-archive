# L-14: One-Shot Verified Builder Front-Door — "type an idea, get an app + its receipt" over the existing pipeline

STATUS: PLANNED (WAVE 3 of L-T2; M; ACTIVATION + COMPOSE)

Objective: give the lazy/one-shot user a single entrypoint — `elai build "<idea>"` (and a one-field dashboard flow) — that turns an idea into a running app AND attaches the trust spine's RRA receipt, so the lazy product IS the trust product with a simple front end (FS-3), not a second dumber product. ELAI already has the whole pipeline: PIPELINE FACTORY L1/L2 [WIRED] (Idea->Concepts->Phases->Steps->BabySteps, deterministic L2 assembly), PRODUCTION FULLSTACK CODEGEN E-12 [WIRED] (7 framework manifests + 15-item checklist) + web templates, J-63 `elai artifact run` [PLANNED] (run the generated app under Tier-C), I-40 `elai wrap` + RRA [WIRED] (the receipt). L-14 is the missing FRONT DOOR: one command that drives the pipeline end-to-end on production defaults, runs the L-11 output-quality gate over the generated code, and emits the app + a verifiable RRA receipt. HARD BOUNDARY: the one-shot output ALWAYS ships the RRA receipt (FS-3); the value claim ("$20=$200" / "more per dollar") stays gated on a measured SAFEDELTA/QDELTA number (KD-1/ID-5), never asserted in the UI. GD-1: do NOT re-author the pipeline factory / E-12 / J-63 / I-40 / J-2 — wire them behind one verb.

Estimated effort: ~1 session (M). In-process + harness; the live app-run + cloud build is the J-63/RunPod concern (a deterministic fixture build ships now, W-10).
Pre-requisites (re-grep live per W-7): PIPELINE FACTORY L1/L2 (the Idea->tree->assembly path); E-12 framework_profile + the 15-item checklist + templates/web/*; J-63 `elai artifact run` + the loopback broker; I-40 `elai wrap` + `elai rra verify` + RunArtifactEmitted; L-11 output-quality gate (the generated code passes it); J-2 ELAI-AS-MCP control plane (the ELAI-as-a-server surface a CC/Codex user wires in); AutonomousGate (the unattended run).
Parallel-safe with: L-11/L-12/L-13 (disjoint); soft-dep L-11 (the quality gate over the generated code) + J-63 (the run seam). If J-63 not shipped, emit the app + receipt without the in-broker run and record residue.
Source: output-quality-and-familiarity-surface-2026-06-27.md PART 2 (FINDING-8) + V-5/V-6 + L-OQF-GROUNDING-MAP TS-19/TS-20/TS-21/TS-22/TS-23 + NN-5 + FS-3/FS-4. Doctrine: L-0_INDEX.md (FS-3, GD-4, ID-5, KD-1).

## Context
Why this matters: the lazy user and the trust user are the SAME engine at two surfaces (the design verdict). The lazy user wants "type an idea, get an app" — which CC/Codex/Lovable-class tools do — but ELAI's wedge is that it can ATTACH a receipt the others cannot: an RRA (reproducible-run artifact) that a third party can verify the build actually ran, was wired, and passed its gates (I-40 + the F-4 cant-lie spine). FS-3 makes the receipt non-optional on the one-shot path — otherwise the one-shot product dilutes into a toy (A-4). The positioning (project_market_positioning_2026 + the GTM ledger): sold as "proven more value per dollar, verified", never "cheap"; the value number is ELAI-measured (SAFEDELTA/QDELTA), never asserted (KD-1/ID-5). The build runs through the SAME output-quality gate (L-11) so a one-shot app is held to the same bar as any ELAI code — the any-model quality story (OQ-5) applies most here, since the lazy user may be on the cheap local lane.

## Existing state
- PIPELINE FACTORY L1/L2 [WIRED]: Research+Designer -> PipelineSpec/QualitySpec, recursive Idea->Concepts->Phases->Steps->BabySteps; deterministic L2 assembly NO LLM, AutoB2G DAG repair. [MAP 1.12]
- PRODUCTION FULLSTACK CODEGEN E-12 [WIRED 06-07]: 7 framework manifests + 15-item checklist + templates/web (vanilla/astro/vite-react/vite-svelte). [MAP 1.12 / .elai_cc/templates/web]
- J-63 LOCAL-LANE CLAUDECEPTION `elai artifact run` [PLANNED]: keyless loopback broker, Tier-C isolation, trust_dialog-gated. [MAP 1.9]
- I-40 TRUST-LAYER WRAPPER + RRA [WIRED]: `elai wrap` + `elai rra verify`, RunArtifactEmitted, kill-switch ELAI_WRAP_RRA_DISABLE. [MAP 1.11]
- J-2 ELAI-AS-MCP control plane [PLANNED]: task_submit/status/... over stdio, same FSM gates. [MAP 1.3]
- L-11 output-quality gate [PLANNED, this track]: the generated code passes it.

## Reads (mandatory before coding)
1. _grounding/L-OQF-GROUNDING-MAP.md (TS-19/20/21/22/23, NN-5, FS-3/FS-4); output-quality-and-familiarity-surface-2026-06-27.md PART 2 + A-4.
2. PIPELINE FACTORY L1/L2 entrypoint; E-12 framework_profile + the 15-item checklist + templates/web; J-63 `elai artifact run`; I-40 `elai wrap`/`elai rra verify`/RunArtifactEmitted; AutonomousGate; J-2 MCP control plane.
3. F-0 W-1; G-0 GD-4; CLAUDE.md ID-5; project_market_positioning_2026 (the "verified not cheap" posture); the GTM ledger (the value-number-gated pitch).

## Tasks
1. NN-5a `elai build "<idea>"` (~70 LOC): one verb that drives PIPELINE FACTORY L1->L2 on production defaults (idea -> tree -> deterministic assembly) under the AutonomousGate, selecting an E-12 framework_profile + template; outputs a generated app dir. Reuse the factory entrypoint, do not re-impl.
2. NN-5b attach the receipt (~30 LOC, FS-3 non-negotiable): wrap the whole build in `elai wrap` so it emits an RRA RunArtifactEmitted; the build output ALWAYS includes the receipt path + `elai rra verify` instructions. The receipt is part of the deliverable, not an opt-in.
3. quality through L-11 (~20 LOC): the generated code passes the L-11 output-quality gate (SOLID-proxy + maintainability + J-23 generated-lane) before the build is reported DONE; on a weak local lane the C-32 MiniCritic + J-55 routing keep it gate-passable (OQ-5). A build that fails the quality gate is reported with the findings, not silently shipped.
4. NN-5c `elai artifact run` integration (~20 LOC): once built, offer to run the app via J-63's loopback broker under Tier-C; if J-63 unshipped, emit the app + receipt and record residue.
5. dashboard one-field flow (~surface, decide at edit): a "build from an idea" entry on the ProjectSelector/Intro screen that shows the tree forming + the receipt + the quality verdict; reuse the existing pipeline/plan windows.
6. expose via J-2 (compose, ~10 LOC): `elai build` reachable as a J-2 MCP verb so a CC/Codex user can drive ELAI-as-a-server (FS-4); do NOT re-author J-2.
7. workflow/reasoning/L-14-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - `elai build "<idea>"` CLI verb (the one-shot front door) + the dashboard one-field flow
  - the receipt-attachment wrapper (build => app + RRA RunArtifactEmitted, FS-3)
consumes_symbols:
  - PIPELINE FACTORY L1/L2 entrypoint (DRIVE, never re-implement); E-12 framework_profile + templates/web; AutonomousGate
  - I-40 `elai wrap` + RunArtifactEmitted + `elai rra verify` (the receipt); L-11 output-quality gate; J-63 `elai artifact run`; J-2 MCP control plane (the server verb)
reachable_from:
  - the CLI > `elai build` > pipeline factory > E-12 codegen > L-11 quality gate > I-40 wrap [PRODUCTION one-shot]
  - the dashboard one-field flow > the same path [PRODUCTION lazy UX]
  - the J-2 MCP server > build verb [PRODUCTION ELAI-as-a-server]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO one-shot output without an RRA receipt (FS-3 — the receipt is the deliverable)
  - NO value/"$X=$Y"/cheaper claim in the UI not backed by a measured SAFEDELTA/QDELTA number (KD-1/ID-5)
  - NO generated app reported DONE that failed the L-11 quality gate (silently)
  - NO re-implementation of the pipeline factory / E-12 / J-63 / I-40 / J-2 (GD-1 P0); blake3 not sha2
frontend_pair:
  a "build from idea" entry on ProjectSelector/Intro + the tree-forming + receipt + quality-verdict view (reuse PlanWindow/pipeline surfaces) + cross_lang_diff PASS, OR justified N/A.
observable_effect:
  trigger: "elai build 'a TODO app with a REST API and 3 routes' (ELAI_SEED, production defaults)"
  expect_ssb: "reuse RunArtifactEmitted (I-40) + the pipeline/plan rows + the L-11 quality verdict; net 0 expected (compose); re-grep LIVE ~268/577 (W-9)"
  expect_artifact: "a generated app dir that passes the L-11 quality gate + an RRA receipt that `elai rra verify` validates (FS-3); the build report includes the receipt path; the deterministic fixture build ships now, the live cloud/local build arm is RunPod-gated (W-10)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-14_one_shot_verified_builder_front_door.md --run-trigger
rg -n "elai build|cmd_build|one_shot_build" .elai_cc/crates   # the verb exists, non-test caller
rg -ci "RunArtifactEmitted|elai wrap|build_rra" <the build path>   # the receipt is attached (FS-3, MUST be present)
rg -ci "cheaper|\\$[0-9]+ ?= ?\\$[0-9]+|more value" .elai_cc/crates/dashboard-v2/src   # any such string MUST be backed by a measured number (KD-1) or absent
test -f workflow/reasoning/L-14-decisions.md
```

## ACCEPTANCE
- [ ] FS-3: every `elai build` output ships an RRA receipt that `elai rra verify` validates; the receipt is non-optional.
- [ ] FS-4: `elai build` is reachable as a J-2 MCP verb (ELAI-as-a-server for a CC/Codex user).
- [ ] the generated code passes the L-11 output-quality gate before DONE; a failing build reports findings, never silently ships.
- [ ] KD-1/ID-5: no value/cheaper claim in the UI unless backed by a measured SAFEDELTA/QDELTA number.
- [ ] GD-1: the pipeline factory / E-12 / J-63 / I-40 / J-2 are driven, not re-implemented. GD-4: ships behind a kill-switch, fixture-first.
- [ ] cargo check --workspace GREEN; auditor PASS on -p. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai build` drives the real pipeline end-to-end and emits an app + a verifiable RRA receipt (not cargo-green).
- W-2: kill-switch on the build verb; reuse ELAI_WRAP_RRA_DISABLE for the receipt path; no new enable graveyard.
- W-3: drives BUILT pipeline/E-12/I-40 code, not a summary-only port.
- W-4: ProjectSelector/Intro frontend_pair + cross_lang_diff, or justified N/A.
- W-5/W-10: the deterministic fixture build ships now; the live cloud/local build + `elai artifact run` arm is RunPod/J-63-gated, separately checkboxed.
- W-7: pipeline factory / E-12 / I-40 / J-63 / J-2 anchors re-grepped at edit.
- W-9: prefer-reuse RunArtifactEmitted + pipeline rows; net 0 expected.
- R10: the off-agent integration smoke drives `elai build` on a HARD idea (multi-route app) end-to-end, not a toy.

## Tests to add
- crates/contracts/tests/l14_one_shot_build_contract.rs (~90 LOC): (a) `elai build` on a fixture idea produces an app dir + an RRA receipt that verifies (FS-3); (b) a build whose generated code fails L-11 is reported with findings, not DONE; (c) the receipt is recomputable/verifiable (GD-2 + I-40 `rra verify`); (d) no value-claim string ships without a measured number (KD-1); (e) the build verb is reachable from the J-2 MCP surface (FS-4).

## Files touched
- CREATE/MODIFY: the `elai build` CLI verb (args.rs + a build_cmd module, ~90 LOC); the receipt-attachment wrapper (~30 LOC); the L-11 quality-gate call (~20 LOC); the J-63 run integration (~20 LOC); the J-2 MCP build verb (~10 LOC)
- MODIFY: ProjectSelector.svelte / Intro.svelte one-field flow (+ Tauri command pair)
- CREATE: crates/contracts/tests/l14_one_shot_build_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-14-decisions.md

## Citations / Grounding
- output-quality-and-familiarity-surface-2026-06-27.md PART 2 FINDING-8 + A-4 + V-5/V-6.
- ELAI ground truth (L-OQF-GROUNDING-MAP TS-19/20/21/22/23, NN-5): PIPELINE FACTORY L1/L2, E-12 + templates/web, J-63 artifact-run, I-40 wrap/RRA, J-2 MCP (re-grepped at edit, W-7).
- CLAUDE.md / memory: project_market_positioning_2026 (verified-not-cheap), the GTM ledger (value-number-gated pitch), ID-5/KD-1, blake3-only.

## Open questions / risks
- J-63 not yet shipped: emit the app + receipt without the in-broker run; the `elai artifact run` integration is a soft-dep, residue named.
- the lazy-user UX vs the trust spine: the receipt must be PRESENT but not in the way — surface it as a one-line "verified: <receipt> (elai rra verify)" not a wall of audit output; decide the UX at edit (FS-3 present, not noisy).
- value claim discipline: the UI must not ship a "$20=$200"/cheaper line until L-10/I-38/G-6 produce a real number (KD-1); seed the UI with the receipt + the resolve-rate, not a dollar multiplier.

## Next
L-14 is the lazy front door over the whole stack, with the trust receipt attached — the bridge between the one-shot user and the trust product. With L-11 (quality), L-12 (skills), L-13 (rules/hooks), L-14 (one-shot) the L-T2 track delivers both audiences on one engine. The measured value claim it gates on is L-10/I-38/G-6 (a separate bench), never asserted.

End of L-14.
