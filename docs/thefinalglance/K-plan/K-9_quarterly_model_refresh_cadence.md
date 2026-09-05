# K-9: quarterly model-refresh cadence — a recurring evaluated checkbox so the local lane never ages silently; candidates vs incumbent on ELAI's own fixtures

STATUS: PLANNED (WAVE KW-2; process + bench wrapper; depends J-9 catalog; the cadence cannot silently swap — user ratifies)

Objective: the local lane ages silently — Qwen2.5-Coder-7B@12GB is below the 2026 ~24GB autonomous floor,
and the useful class is now efficient-MoE 30B-A3B on 24GB (KS-2 §1.9; KS-3 report 17). This session ships
a recurring (quarterly) evaluated checkbox: top catalog candidates per lane vs the incumbent on ELAI's
OWN fixtures (pass-rate, VRAM, tok/s, $/verified-task), riding J-9's catalog + the E-27 AWQ and J-19 vLLM
lanes; output = an ADOPT/KEEP/RE-EVALUATE decision row the user ratifies. It also re-states the lane
doctrine. Hard boundaries: the cadence CANNOT silently swap a model (a regressing candidate is auto-KEPT-
incumbent; a swap needs user ratification — GD-4/user-gate); prior rejections (Gemma-4, Qwen3-32B) are
already lifted to tier-conditional CATALOG data by J-9 — routing stays untouched without user sign-off.

Estimated effort: ~1 session (~4-5 h). The cadence is a checklist + a bench wrapper over J-9's catalog;
the live model-eval legs are RunPod/2x3090-gated (W-10).
Pre-requisites: J-9 hardware model catalog (A-7 — HardwareModelCatalogEvent; the catalog substrate). E-27
AWQ lane, J-19 vLLM backend, E-31 tier matrix, I-16 KV eval. The 2x3090 box for the 24GB-class autonomy
candidates (pending — KS-2 §1.9).
Parallel-safe with: K-1, K-7, K-8 (disjoint). Depends J-9.
Source: K-MASTER-BRIEF §3.2 K-9 [KS-1]; KS-2 §1.9, KS-3 report 17; grounding map (A-7/J-9). Doctrine: K-0_INDEX.md.

## Context
Why this matters: the local lane's scope is mechanical/privacy/latency (H-50 doctrine), and a silently
stale incumbent undermines even that; the 24GB-class MoE candidates are the autonomy tier the 2x3090 box
will host. Mechanism: a quarterly evaluated checkbox runs the top catalog candidates per lane (coding/
vision/OCR/embedding) against the incumbent on ELAI's own fixtures (pass-rate, VRAM, tok/s,
$/verified-task), riding J-9's catalog; the output is an ADOPT/KEEP/RE-EVALUATE decision row the user
ratifies — the cadence never swaps on its own. External: KS-2 §1.9 (the ~24GB floor, the MoE-30B-A3B
trend, open-weight SWE-bench 70s vs frontier ~80) is [EXTERNAL/DESIGN] context; the ELAI verdict is
fixture-measured (HD-1).

## Existing state (A/B..J coverage)
- A-7 J-9 hardware model catalog: HardwareModelCatalog + HardwareModelCatalogEvent (the per-lane
  eligibility catalog off the shipped probe stack; CatalogBuilt at startup). The cadence rides it.
- E-27 AWQ lane (the never-built checkpoint J-19 also references), J-19 vLLM-V1 backend (the additive
  backend), E-31 tier matrix (the lane definitions), I-16 KV eval.
- H-50 lane doctrine: local = mechanical/privacy/latency lane; the cadence re-states it.
- Prior rejections (Gemma-4, Qwen3-32B) lifted to tier-conditional catalog data by J-9 (HW-14/DC-3 in the
  J-perception grounding map) — informational-only; routing untouched.
- memory project_vram_tier_plan (the per-VRAM configs + the 2x3090-coming note).

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (A-7 / J-9 anchors) + the J-9 plan file (the catalog shape).
2. J-9's HardwareModelCatalog + the model_catalog.toml (the candidate source) — re-grep live (W-7).
3. E-27 AWQ + J-19 vLLM + E-31 tier matrix (the lanes the cadence evaluates).
4. The SAFEDELTA/SeCodePLT fixture set (the ELAI fixtures the candidates are scored on — pass-rate +
   $/verified-task).
5. memory project_vram_tier_plan + the H-50 lane doctrine.

## Tasks (WAVE KW-2 — cadence + wrapper)
1. Author the cadence checklist (docs/strategy/ELAI-MODEL-REFRESH-CADENCE.md or a recurring workflow doc):
   quarterly, per lane, top catalog candidates vs incumbent on ELAI fixtures; the decision row schema
   (ADOPT/KEEP/RE-EVALUATE + the measured pass-rate/VRAM/tok-s/$-per-verified-task). Re-state the lane
   doctrine (local = mechanical/privacy/latency H-50; 24GB-class = autonomy candidate; cloud = planning). ~80 lines.
2. Build a thin bench wrapper over J-9's catalog (benchmarks/model_refresh_runner.rs or a CLI arm): given
   the catalog candidates per lane, run them against the ELAI fixtures, emit the decision row. The live
   model-eval legs are RunPod/2x3090-gated (W-10); the deterministic catalog-read + decision-schema leg
   ships now. ~150-200 LOC.
3. SSB: prefer-reuse J-9 HardwareModelCatalogEvent rows; else +ModelRefreshCadence (Pattern-2; Kind:
   CandidateEvaluated / DecisionRecorded / IncumbentKept). Re-grep 236/545 at edit; sequential bump.
4. SAFETY: a candidate that REGRESSES fixtures is auto-KEPT-incumbent; a swap needs user ratification
   (GD-4/user-gate). Routing/allowlist/CLAUDE.md untouched without user sign-off.
5. NEGATIVE arms: a candidate claiming a win without a fixture run FAILS; a swap without ratification is blocked.
6. workflow/reasoning/K-9-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - docs/strategy/ELAI-MODEL-REFRESH-CADENCE.md (the recurring checklist + decision-row schema + lane doctrine)
  - the model-refresh bench wrapper (benchmarks/model_refresh_runner.rs or a CLI arm)
  - +ModelRefreshCadence SSB (or reused J-9 HardwareModelCatalogEvent rows — decide at edit)
consumes_symbols:
  - J-9 HardwareModelCatalog + model_catalog.toml (A-7 — the candidate source; re-grep, never re-implement)
  - E-27 AWQ + J-19 vLLM + E-31 tier matrix (the lanes evaluated)
  - the SAFEDELTA/SeCodePLT fixtures (the ELAI scoring fixtures)
reachable_from:
  - `elai bench model-refresh [--lane <lane>]` (or the recurring checklist invocation) [PRODUCTION CLI]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO silent model swap (a regressing candidate is auto-KEPT-incumbent; a swap needs user ratification)
  - NO routing/allowlist/CLAUDE.md mutation without user sign-off
  - NO re-implementation of J-9's catalog (compose it; GD-1)
  - NO candidate "win" without a fixture run (the decision row needs measured numbers; HD-1)
  - NO .unwrap()/.expect() on the prod wrapper path
frontend_pair:
  a refresh-decision tile (the ADOPT/KEEP/RE-EVALUATE row per lane) on a runtime/catalog panel (decide
  at edit, W-4) OR W-6-justified CLI-only absence.
observable_effect:
  trigger: "elai bench model-refresh --lane coding   (the first cadence run)"
  expect_ssb: "ModelRefreshCadence (CandidateEvaluated + DecisionRecorded/IncumbentKept) — or reused J-9
    rows — recomputable from the journal (GD-2)"
  expect_artifact: "the first cadence run produces the decision row + a dated refresh-report; a candidate
    that REGRESSES fixtures is auto-KEPT-incumbent (the NEGATIVE/safe arm: the cadence cannot silently swap)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-9_quarterly_model_refresh_cadence.md --run-trigger
test -f docs/strategy/ELAI-MODEL-REFRESH-CADENCE.md
rg -n "model-refresh|model_refresh" crates/cli/src/   # the wrapper entrypoint exists
rg -n "IncumbentKept|auto.kept|ratif" crates/orchestrator/src/benchmarks/model_refresh_runner.rs   # the no-silent-swap safety
rg -c "routing|allowlist" crates/orchestrator/src/benchmarks/model_refresh_runner.rs   # MUST be 0 (routing untouched)
test -f workflow/reasoning/K-9-decisions.md
```

## ACCEPTANCE
- [ ] the cadence cannot silently swap: a regressing candidate is auto-KEPT-incumbent; a swap needs user ratification (GD-4).
- [ ] routing/allowlist/CLAUDE.md untouched without user sign-off.
- [ ] GD-1: composes J-9's catalog + E-27/J-19/E-31; re-implements none.
- [ ] HD-1: every candidate verdict is fixture-measured, not a literature figure.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli. SSB net 0..+1, sequential.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the first cadence run produces a decision row + a dated report (not "the doc exists").
- W-2: N/A. W-3: N/A. W-4: refresh-decision tile or W-6-justified absence.
- W-5/W-10: the live model-eval legs are RunPod/2x3090-gated; the catalog-read + decision-schema leg now.
- W-7: J-9 catalog re-grepped at edit. W-9: SSB re-read. GD-1/GD-4/HD-1: as ACCEPTANCE.

## Tests to add
- crates/contracts/tests/k9_model_refresh_contract.rs (~80 LOC): (a) the cadence reads J-9's catalog
  candidates; (b) it produces an ADOPT/KEEP/RE-EVALUATE decision row with measured fields; (c) a
  regressing candidate is auto-KEPT-incumbent (R2 safe arm); (d) a candidate without a fixture run FAILS;
  (e) no routing/allowlist symbol is mutated (the grep-isolation assertion).

## Files touched
- CREATE: docs/strategy/ELAI-MODEL-REFRESH-CADENCE.md (~80 LOC)
- CREATE: crates/orchestrator/src/benchmarks/model_refresh_runner.rs (~180 LOC)
- MODIFY: crates/cli/src/args_bench.rs + a bench arm (~40 LOC: the `model-refresh` sub)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (IF +ModelRefreshCadence; sequential)
- CREATE: crates/contracts/tests/k9_model_refresh_contract.rs (~80 LOC)
- CREATE: workflow/reasoning/K-9-decisions.md (~30 LOC)

## Citations / Grounding
- KS-2 §1.9: ~24GB useful local floor (4-bit ~24B), MoE-30B-A3B trend, open-weight SWE-bench 70s vs
  frontier ~80; hybrid-routing production evidence thin. [EXTERNAL/DESIGN]
- KS-3 report 17: open-weight/local lane economics; the durable form is workload-specific choices.
- ELAI ground truth: A-7/J-9 (the catalog), E-27/J-19/E-31, the SAFEDELTA/SeCodePLT fixtures, H-50 lane doctrine.
- CLAUDE.md / memory: project_vram_tier_plan (2x3090 coming), No-Gemma-4 (the rejection lifted to
  catalog-data-only by J-9, not resurrected as a default), GD-4/HD-1.

## Open questions / risks
- The 2x3090 box may not materialize (KS-2 kill-switch): the cadence still runs (catalog + cloud-lane
  comparisons); 24GB-class adoption deferred, lane doctrine unchanged.
- Cadence cost: a full per-lane model bench is expensive — keep the fixture set small + the live legs
  RunPod-gated; the catalog-read + decision-schema leg is the cheap always-runnable part.
- The Gemma-4/Qwen3-32B catalog data is informational-only (J-9 HW-14/DC-3); the cadence must NOT
  surface them as default-adopt candidates without the user's explicit sign-off (memory No-Gemma-4).

## Next
This closes the depreciation/refresh track: K-8 stops building what providers eat, K-9 stops the local
lane aging silently. Together they install the discipline that keeps the corpus from drifting stale.
The K-plan keystones (A-1..A-4 + K-3) ship first; this cadence runs alongside and quarterly thereafter.

End of K-9.
