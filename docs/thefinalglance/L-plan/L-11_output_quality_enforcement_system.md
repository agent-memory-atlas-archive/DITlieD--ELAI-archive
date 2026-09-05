# L-11: Output-Quality Enforcement System — SOLID-proxy + the 4-seam layered gate over EVERY model's code [KEYSTONE]

STATUS: PLANNED (WAVE 1 of L-T2; L; BUILD + COMPOSE; KEYSTONE — the output-quality spine the rest of L-T2 assumes)

Objective: make every piece of code ELAI produces — on ANY backend (cloud Sonnet, local Qwen-7B, BYOK, free-pool, CLI-bridge) — pass a maintainability/SOLID-class quality bar, enforced like the other rules. ELAI has NO by-name SOLID enforcement (grep-confirmed) but already ships the machinery: a generation-time loop (C-32 SkeletonGenerator>MiniCritic>StructuralRetry), a plan-time shape budget (C-31 FilePlan, MAX_FUNCTIONS_PER_FILE=20), an auditor maintainability checker (maintainability_check.py MR-1..MR-4), and two PLANNED gates over generated code (J-65 power-of-ten, J-23 review-catalog). L-11 adds the ONE net-new checker (.claude/tools/solid_proxy_check.py) + the ONE net-new generation-time predicate (a god-object MiniCritic METRIC) and wires output-quality across FOUR seams with role-correct placement. HARD BOUNDARY: the authoritative VERDICT is AUDITOR-owned and frozen (OQ-1, never executor-self — Berkeley RDI in-boundary gamed ~100%); the gen-time MiniCritic is self-CORRECTION (retry), not the verdict. GD-1: COMPOSE C-31/C-32/maintainability_check.py/J-65/J-23/B-29/elai.map/rules-3F/VibeGate-F-1 — re-implementing any is a P0.

Estimated effort: ~1 session (L). The checker + the predicate are deterministic + in-process; no model call; no RunPod. The measured-delta arm composes I-38 QDELTA (its own bench session).
Pre-requisites (re-grep live per W-7): C-31 file_plan.rs (FilePlan, MAX_FUNCTIONS_PER_FILE=20); C-32 mini_critic.rs + structural_retry.rs + the task_runner baby-step dispatch seam; maintainability_check.py + gate-manifest.txt; B-29 impact API (fan-in/fan-out); elai.map TYPE_REF/IMPL_TRAIT tags; rules-engine 3F lifecycle; VibeGate F-1 evaluate_vibegate PreToolUse chain; J-65 power_of_ten_check.py (sibling, may land same wave); J-23 review-catalog (the generated-multilang lane).
Parallel-safe with: L-12/L-13/L-14 (disjoint file scope; L-T2 familiarity files touch skills/rules/pipeline, not the quality gate).
Source: output-quality-and-familiarity-surface-2026-06-27.md PART 1 + V-1/V-2 + L-OQF-GROUNDING-MAP TS-1..TS-12 + NN-1/NN-2 + OQ-1..OQ-5. Doctrine: L-0_INDEX.md (OQ-1..OQ-5, GD-4, TD-3, ID-5, R8).

## Context
Why this matters: "compiles + wired + not-a-stub" still ships a 1200-line god-file or a fn with eight bolted-on conditionals — exactly the entropy an LLM coder adds by taking the shortest path that makes the test green. maintainability-review.md already polices this on ELAI's OWN commits; L-11 turns the same axis on GENERATED code and adds SOLID proxies. The design rule (OQ-1) is load-bearing: an agent grading its own output rubber-stamps, so the verdict is off-agent + frozen (R8); the gen-time loop is allowed because it RETRIES (self-corrects) and is not the verdict. The "any model" goal (OQ-5) is why SEAM 2 (generation-time) is the workhorse: it is the only layer that catches AND FIXES a weak local model's output in-loop — paired with J-55 routing the weak lane to constrained generation that is gate-passable by construction. SOLID is mechanizable only in part (OQ-4): SRP/DIP -> deterministic proxies (god-object via B-29 fan-out, cross-layer-concrete via elai.map); OCP/LSP -> cross-family ADVISORY (J-89), never a deterministic block. Thresholds are ELAI-measured (L-10/I-38), never SOLID/P10 paper constants (ID-5).

## Existing state (the four seams already have hosts)
- C-31 FilePlan [PARTIAL]: per-file max_lines + MAX_FUNCTIONS_PER_FILE=20 at plan time; MistakeRecorder unwired. SEAM 1 host. [MAP/C-SUMMARY]
- C-32 MiniCritic [WIRED]: PASS/WARN/BLOCK per fill + StructuralRetry; mini_critic_verdict SSB. SEAM 2 host. [C-SUMMARY]
- maintainability_check.py [WIRED, frozen R8]: MR-1 file<1000 / MR-2 as-any / MR-3 fn<80 / MR-4 branching; auditor-owned. SEAM 3 floor + SEAM 4 sibling. [CLAUDE.md]
- J-65 power_of_ten_check.py [PLANNED, observe-only]: P10 over generated code; auditor-owned. SEAM 4 sibling. [MAP 1.6]
- J-23 review-catalog [PLANNED]: generated multi-language lane (vulture/knip/deptrac/a11y). SEAM 4 generated lane. [MAP 1.8]
- B-29 impact API [WIRED] + elai.map [WIRED]: the SRP fan-out + DIP/ISP data sources. [MAP 1.1 / CLAUDE.md]
- rules-engine 3F [WIRED]: SHADOW->ACTIVE lifecycle for promoting a quality rule. [MAP 1.7]

## Reads (mandatory before coding)
1. _grounding/L-OQF-GROUNDING-MAP.md (TS-1..TS-12, NN-1/NN-2, OQ-1..OQ-5); output-quality-and-familiarity-surface-2026-06-27.md PART 1 + PART 3 A-1/A-2.
2. .claude/rules/maintainability-review.md (the 8 dimensions + ROLE BOUNDARY + the MR checker + gate-manifest R8); .claude/tools/maintainability_check.py + power_of_ten_check.py (sibling shape); C-32 mini_critic.rs + structural_retry.rs; C-31 file_plan.rs; B-29 impact API; elai.map header (TYPE_REF/IMPL_TRAIT).
3. F-0 W-1; G-0 GD-2/GD-4; CLAUDE.md TD-3/ID-5; the rules-engine 3F lifecycle (SHADOW->ACTIVE).

## Tasks (the 4 seams + the promotion path)
1. NN-1 solid_proxy_check.py (~180 LOC, .claude/tools/, stdlib-only): consume B-29 fan-out + elai.map TYPE_REF/IMPL_TRAIT over the changeset. SRP god-object = (fan-out > K_FANOUT AND (method-count > K_METHODS OR lines > K_LINES)) -> BLOCK class. DIP = a downstream-crate concrete TYPE_REF to an upstream impl where a trait exists -> WARN. ISP = trait with > K_TRAIT_METHODS where the median impl uses < half -> WARN. OCP/LSP NOT covered (advisory only, named in --json residue). Exit 1 on BLOCK / 2 on scan-config error; --files/--base HEAD/--json; tags C-MAINTAINABILITY-SRP/-DIP/-ISP. R5: sound reference (B-29/SCIP), never leaf-name grep.
2. NN-2 the god-object MiniCritic METRIC predicate (~50 LOC in C-32 mini_critic.rs): add a METRIC check (fan-out + method-count of the filled symbol) returning WARN/BLOCK so StructuralRetry re-rolls in-loop; pair with J-55 (route the weak lane to constrained gen). OQ-5. Default WARN (observe-only) until the FP bound clears.
3. SEAM 1 extend C-31 (~30 LOC): a per-file responsibility budget field on FileSpec (max_distinct_concerns, default generous + observe-only) validated at plan time; the SRP-prevention rung (cheapest). No schema bump (serde default like FILE_PLAN).
4. SEAM 3 write-time floor (~20 LOC): route the MR-1/MR-2 DETERMINISTIC subset through the VibeGate F-1 PreToolUse chain as a WARN (BLOCK only on MR-1 crossing / MR-2 BLOCK class). SOLID is NOT a hard PreToolUse block (OQ-2, thrash guard A-1).
5. SEAM 4 auditor authority (~40 LOC): the auditor runs solid_proxy_check.py + maintainability_check.py + (when present) power_of_ten_check.py over the changeset AND the J-23 lane over GENERATED code; freeze solid_proxy_check.py in gate-manifest.txt (R8). The verdict is auditor-owned (OQ-1).
6. the promotion path (~30 LOC): register the SOLID proxies as rules-engine 3F entries in SHADOW (observe-only); promote god-object to BLOCK-default only on fpr<5%/tpr>80% over >=3 real runs (OQ-3/GD-4/J-43). DIP/ISP stay WARN; OCP/LSP stay J-89 advisory.
7. compose the measured number: a `--quality` arm on the I-38 QDELTA runner (gate-ON-vs-OFF maintainability/SOLID-violation rate), ELAI-measured, never a paper constant (ID-5). Do NOT fork the runner.
8. workflow/reasoning/L-11-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - .claude/tools/solid_proxy_check.py — SRP god-object BLOCK + DIP/ISP WARN over the changeset (frozen R8)
  - crates/orchestrator (or crates/tools) MiniCritic METRIC predicate — the god-object gen-time check feeding StructuralRetry
  - C-31 FileSpec responsibility-budget field (plan-time SRP rung)
consumes_symbols:
  - B-29 impact API (fan-in/fan-out — the SRP/DIP source; GENERALIZE, never re-implement)
  - elai.map TYPE_REF/IMPL_TRAIT (DIP/ISP); maintainability_check.py (the floor, reused); power_of_ten_check.py (J-65 sibling); J-23 review-catalog (generated lane)
  - C-32 mini_critic.rs + structural_retry.rs (the gen-time loop); C-31 file_plan.rs; VibeGate F-1 evaluate_vibegate (SEAM 3); rules-engine 3F lifecycle; I-38 QDELTA runner; J-55 modality router
reachable_from:
  - the auditor changeset gate > solid_proxy_check.py [PRODUCTION audit authority]
  - the C-32 baby-step dispatch > MiniCritic METRIC predicate [PRODUCTION gen-time]
  - the VibeGate PreToolUse chain > MR floor [PRODUCTION write-time]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO executor-self quality verdict (OQ-1 — auditor-owned + frozen R8)
  - NO hard PreToolUse SOLID block (OQ-2 — only the MR-1/MR-2 deterministic floor at write-time)
  - NO god-object BLOCK-by-default before the FP bound on >=3 runs (OQ-3/GD-4)
  - NO deterministic OCP/LSP checker (OQ-4 — advisory only); NO leaf-name-grep call graph (R5 — use B-29/SCIP)
  - NO re-implementation of C-31/C-32/maintainability_check.py/B-29 (GD-1 P0); NO paper-constant threshold (ID-5); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  a quality-verdict tile on the vibesec/audit panel (GatesPanel precedent; decide at edit, W-4) OR "N/A — verdicts flow to the existing audit/maintainability surface. Justified."
observable_effect:
  trigger: "run the auditor over a changeset containing a 1200-line god-module (high fan-out + 25 methods) AND a clean 200-line module (real audit seam)"
  expect_ssb: "reuse mini_critic_verdict / a maintainability row OR +1 OutputQualityVerdict (Pattern-2; Kind SrpBlock/DipWarn/IspWarn/PassClean); baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the god-module is flagged C-MAINTAINABILITY-SRP (BLOCK in ACTIVE, WARN in SHADOW); the clean module passes; both recomputable from the changeset under ELAI_SEED (GD-2); and a generated weak-model file with a god-object retries in-loop via StructuralRetry (OQ-5)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md --run-trigger
python .claude/tools/solid_proxy_check.py --files <a known god-object .rs> --json   # exit 1, SRP BLOCK present
python .claude/tools/solid_proxy_check.py --files <a clean .rs> --json              # exit 0
rg -n "solid_proxy_check.py" .claude/gate-manifest.txt                              # frozen (R8)
rg -ci "ocp|lsp.*block|liskov.*block" .claude/tools/solid_proxy_check.py           # MUST be 0 (OQ-4 advisory only)
test -f workflow/reasoning/L-11-decisions.md
```

## ACCEPTANCE
- [ ] OQ-1: the authoritative SOLID/maintainability verdict is auditor-owned + frozen in gate-manifest (R8); the executor never self-grades.
- [ ] OQ-2: write-time has ONLY the MR-1/MR-2 deterministic floor; SOLID proxies live at audit-time (+ gen-time WARN).
- [ ] OQ-3: god-object ships SHADOW/observe-only; promotes to BLOCK-default only on fpr<5%/tpr>80% over >=3 runs.
- [ ] OQ-4: SRP/DIP are deterministic proxies; ISP weak-WARN; OCP/LSP are J-89 cross-family advisory, NOT a deterministic block (residue named).
- [ ] OQ-5: a generated weak-model god-object retries in-loop via the C-32 MiniCritic METRIC + StructuralRetry (the any-model leg).
- [ ] R5: the call graph comes from B-29/SCIP, never leaf-name grep; TD-3: the LLM never adjudicates a quality verdict.
- [ ] GD-1: C-31/C-32/maintainability_check.py/B-29 composed, not re-implemented. ID-5: thresholds ELAI-measured, never SOLID/P10 constants.
- [ ] cargo check --workspace GREEN; auditor PASS on the affected crates (-p). No new dependency (stdlib-only checker).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the auditor seam actually runs solid_proxy_check.py over a real changeset and emits a verdict (not cargo-green / file-exists).
- W-2: kill-switch not enable-flag — the SOLID proxies are rules-engine SHADOW entries (lifecycle-gated), the MR floor is already live; no new default-OFF enable graveyard.
- W-3: extends BUILT C-32/maintainability_check.py, not a summary-only port.
- W-4: frontend_pair decided at edit or justified N/A.
- W-5/W-10: in-process deterministic; no RunPod; the I-38 QDELTA live arm is QDELTA's own RunPod concern.
- W-7: C-32/C-31/maintainability_check.py/B-29/elai.map anchors re-grepped at edit.
- W-9: prefer-reuse mini_critic_verdict / maintainability rows; +0..1 from LIVE baseline.
- R8: solid_proxy_check.py frozen in gate-manifest.txt after human review (gate_freeze_check.py --lock).
- GD-2/ID-5: every flag is recomputable from the changeset under ELAI_SEED; thresholds ELAI-measured.

## Tests to add
- crates/contracts/tests/l11_output_quality_contract.rs (~100 LOC): (a) the MiniCritic METRIC flags a god-object fill and StructuralRetry re-rolls (OQ-5); (b) a clean fill passes; (c) the C-31 responsibility budget rejects an over-concern FileSpec at plan time; (d) the verdict path is auditor-only — an executor-context call is rejected/ignored (OQ-1); (e) thresholds are config/measured, not literals (ID-5).
- .claude/tools/ self-test fixtures for solid_proxy_check.py: a god-object .rs (exit 1 SRP), a clean .rs (exit 0), a cross-layer-concrete .rs (DIP WARN), an OCP/LSP case (NO block — residue).

## Files touched
- CREATE: .claude/tools/solid_proxy_check.py (~180 LOC) + fixtures
- MODIFY: .elai_cc/crates/.../mini_critic.rs (~50 LOC METRIC predicate); C-31 file_plan.rs FileSpec (~30 LOC); the VibeGate F-1 PreToolUse chain (~20 LOC MR floor); the auditor changeset seam (~40 LOC); rules-engine seed rows (~30 LOC); .claude/gate-manifest.txt (+1 frozen entry)
- MODIFY: the I-38 QDELTA runner (~20 LOC --quality arm) — compose, do not fork
- CREATE: crates/contracts/tests/l11_output_quality_contract.rs (~100 LOC)
- CREATE: workflow/reasoning/L-11-decisions.md

## Citations / Grounding
- DS-1 SOLID (R.C. Martin 2000) [EXTERNAL/DESIGN-RATIONALE, ID-5 — taxonomy only, no number]; DS-2 Holzmann Power-of-Ten (NASA/JPL 2006, carried by J-65). output-quality-and-familiarity-surface-2026-06-27.md PART 1.
- ELAI ground truth (L-OQF-GROUNDING-MAP TS-1..TS-12, NN-1/NN-2): C-31 file_plan.rs, C-32 mini_critic.rs/structural_retry.rs, maintainability_check.py MR-1..MR-4, J-65, J-23, B-29, elai.map, rules-3F, VibeGate F-1, I-38 (re-grepped at edit, W-7).
- CLAUDE.md: maintainability-review.md ROLE BOUNDARY (OQ-1), TD-3, ID-5, no .unwrap, blake3-only, gate-manifest R8.

## Open questions / risks
- The fan-out/method-count thresholds for god-object: ELAI-measured at L-10/I-38 (ID-5), seeded conservatively + SHADOW-first (OQ-3). Decide the seed at authoring; never a SOLID/P10 constant.
- SRP fan-out false-positives on legitimately-central modules (an orchestrator IS high fan-out): the BLOCK requires fan-out AND size; central-but-small modules WARN at most; the >=3-run FP bound (OQ-3) is the safety net. If FP stays high, god-object stays WARN permanently (A-1).
- Generated TS/Svelte/Python: the SOLID-proxy is Rust-strongest; for generated TS/Py the J-23 catalog (vulture/knip/deptrac) does the real work (GD-5 residue, named).

## Next
L-11 is the output-quality spine. L-12/L-13/L-14 are the familiarity surface and are independent of it. The measured proof is the I-38 QDELTA `--quality` arm (its own bench session); the SOLID proxies promote SHADOW->ACTIVE only when that delta + the FP bound clear.

End of L-11.
