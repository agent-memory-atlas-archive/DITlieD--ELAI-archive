# K-5: risk-tiered gating policy — one R8-frozen GateIntensity table routing WHICH verification legs fire per risk tier; collapse where safe, gate where expensive

STATUS: PLANNED (WAVE KW-3; BUILD policy + dispatch; KD-3/R8; depends K-4 — the ceiling it must fit under)

Objective: adaptive gating beats gate-everything (KS-3 report 17); ELAI has the risk substrate (B-22 risk
levels, C-30 complexity classes, J-16 cones, F-1 compound gate) but no single POLICY that routes which
verification legs fire per risk tier. This session BUILDS a deterministic, R8-FROZEN GateIntensity policy
table: cheap sub-ms deterministic checks ALWAYS-ON (argv allowlist E-30, secrets/dep gates F-1/F-9, taint
G-1); expensive legs (mutation I-20, differential H-26, VLM tier-2, formal E-53) fire on RISK TRIGGERS —
blast-radius cone size (J-16), CWE-surface class (F-2), autonomy level (B-22), novelty (D-15), file
criticality. Hard boundaries: the LLM NEVER picks the tier (TD-3); the policy table's hash is in the
frozen gate-manifest (a run cannot weaken its own gating — R8); and the measured composed tax must fit
under K-4's locked ceiling on >=3 real runs (GD-4/KD-3).

Estimated effort: ~1 session (~4-5 h). In-process (a deterministic selector over existing risk signals).
Pre-requisites: K-4 (the composed ceiling the policy must fit under). B-22 risk levels, C-30 complexity
classes, J-16 cones (BlastRadiusEvent), F-1 compound gate, F-9/G-1/E-30 always-on legs, I-20/H-26/E-53
expensive legs, D-15 novelty. The frozen gate-manifest (.claude/gate-manifest.txt, R8).
Parallel-safe with: K-6, K-9 (disjoint). DEPENDS on K-4.
Source: K-MASTER-BRIEF §3.2 K-5 [KS-1]; KS-3 report 17, KW-4; grounding map KG-8, KD-3/R8. Doctrine: K-0_INDEX.md.

## Context
Why this matters: gate-everything is unaffordable (K-4 proves the composed cost) and the research's
remedy is risk-adaptive gating (collapse where safe, gate where expensive — KW-4's CodeAct collapse
finding applied to verification). Mechanism: a deterministic policy table maps a risk signal vector
(cone size, CWE class, autonomy, novelty, file criticality) to a GateIntensity (which legs fire); a
trivial rename fires only the always-on set, a high-cone auth-touching edit fires the full stack; the LLM
proposes nothing (TD-3); the table is frozen (R8). The policy is validated by measuring its composed tax
under K-4's ceiling. External: KW-4 (collapse-where-safe) is [DESIGN] context.

## Existing state (A/B..J coverage)
- B-22 risk levels (LOW/MEDIUM/HIGH), C-30 complexity classes — the risk substrate (PLAN-asserted symbols;
  re-grep at edit, W-7).
- J-16 BlastRadiusEvent cone (ConeComputed) — the blast-radius signal.
- F-1 compound gate + F-9 dep gate + E-30 argv allowlist + G-1 taint — the always-on cheap legs.
- I-20 mutation, H-26 differential, E-53 formal (Verus/Dafny), VLM tier-2 — the expensive legs.
- D-15 novelty signal; file criticality (DKG/B-29 centrality).
- KG-8 wire-dark-allowlist.toml — the gate inventory (which legs exist to tier).
- K-4 the composed ceiling (the budget this policy must fit under).
- TD-3 (LLM never adjudicates a gate decision) — binds the tier selection.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KG-8, KD-3/R8) + K-4's plan file (the ceiling contract).
2. The risk-signal sources: B-22 risk-level + C-30 complexity + J-16 cone (BlastRadiusEvent) + F-2 CWE
   class + D-15 novelty + B-29 centrality (re-grep the live symbols, W-7).
3. F-1 compound gate (the existing always-on stack) + the expensive-leg entrypoints (I-20/H-26/E-53/VLM).
4. .claude/gate-manifest.txt (R8 — where the policy-table hash is frozen) + .claude/tools/gate_freeze_check.py.
5. CLAUDE.md TD-3 (LLM never picks the tier) + GD-4/KD-3 (the FP/friction bound).

## Tasks (WAVE KW-3 — the policy)
1. Define the GateIntensity policy table (a deterministic data table: risk-signal vector -> the set of
   legs that fire). Always-on tier = E-30/F-1/F-9/G-1; risk-triggered tiers add I-20/H-26/E-53/VLM by
   trigger. ~120 LOC + the table data.
2. Build the selector/dispatcher: read the risk signals for an edit, look up the GateIntensity, fire the
   selected legs. The LLM never picks the tier (TD-3 — the selector is deterministic over signals). ~120 LOC.
3. FREEZE the policy table: add its hash to .claude/gate-manifest.txt (R8); gate_freeze_check.py BLOCKs a
   run that edited its own policy table.
4. SSB: +GateIntensitySelected (Pattern-2; Kind: TierAssigned / ExpensiveLegFired / CollapsedToCheap) OR
   reuse F-1 compound-gate rows. Re-grep 236/545 at edit; sequential bump.
5. VALIDATE under K-4: measure the composed tax of the policy on >=3 real runs; it must fit under K-4's
   ceiling (GD-4/KD-3).
6. NEGATIVE arms: an attempt to edit the frozen table inside a run is BLOCKED; an LLM-proposed tier is ignored.
7. workflow/reasoning/K-5-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/gate_intensity.rs (the policy table + the deterministic selector/dispatcher)
  - the policy-table hash row in .claude/gate-manifest.txt (R8 freeze)
  - +GateIntensitySelected SSB (or reused F-1 rows — decide at edit)
consumes_symbols:
  - B-22 risk levels + C-30 complexity + J-16 BlastRadiusEvent cone + F-2 CWE class + D-15 novelty +
    B-29 centrality (the risk-signal vector — re-grep; never re-implement)
  - F-1/F-9/E-30/G-1 (always-on legs) + I-20/H-26/E-53/VLM (expensive legs) — the legs it routes, never
    re-implements
  - K-4's composed ceiling (the budget the policy must fit under)
  - .claude/tools/gate_freeze_check.py (R8 enforcement of the frozen table)
reachable_from:
  - the auditor's per-edit gate-dispatch path consults gate_intensity::select [PRODUCTION audit gate]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-chosen tier (the selector is deterministic over signals; TD-3)
  - NO unfrozen policy table (its hash is in the gate-manifest; R8)
  - NO re-implementation of any risk signal or any gate leg (compose; GD-1)
  - NO gate-everything default (the always-on tier is the cheap set only; KD-3)
  - NO .unwrap()/.expect() on the prod selector path
frontend_pair:
  tier-decision rows on the audit-timeline surface (decide at edit, W-4) OR W-6-justified absence.
observable_effect:
  trigger: "the auditor runs an edit batch through the gate-dispatch path (a trivial rename vs a
    high-cone auth-touching edit)"
  expect_ssb: "GateIntensitySelected (TierAssigned + ExpensiveLegFired/CollapsedToCheap) — or reused
    F-1 rows — showing zero expensive-leg rows for the rename and the full stack for the auth edit"
  expect_artifact: "the policy-table hash in the frozen manifest; the measured composed tax under K-4's
    ceiling on >=3 runs; the NEGATIVE arm: editing the frozen table inside a run is BLOCKED (R2/R8)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-5_risk_tiered_gating_policy.md --run-trigger
python .claude/tools/gate_freeze_check.py    # the policy table is frozen; a run that edited it is BLOCK
rg -n "GateIntensity|select" crates/orchestrator/src/gate_intensity.rs   # the deterministic selector exists
rg -c "llm|model.*tier|tier.*llm" crates/orchestrator/src/gate_intensity.rs   # MUST be 0 (TD-3: LLM never picks)
rg -n "gate_intensity" .claude/gate-manifest.txt   # the table hash is frozen (R8)
test -f workflow/reasoning/K-5-decisions.md
```

## ACCEPTANCE
- [ ] KD-3: the policy fits under K-4's composed ceiling on >=3 real runs (GD-4).
- [ ] TD-3: the tier is selected deterministically over risk signals; the LLM never picks it.
- [ ] R8: the policy table's hash is in the frozen gate-manifest; a run editing it is BLOCKED.
- [ ] GD-1: composes the risk signals + the gate legs; re-implements neither.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. SSB net 0..+1, sequential.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the auditor's gate-dispatch consults the selector + the journal shows tier-appropriate legs.
- W-2: N/A. W-3: N/A. W-4: tier-decision rows or W-6-justified absence.
- W-5: real edit-batch trigger. W-7: risk-signal + gate-leg anchors re-grepped at edit. W-9: SSB re-read.
- GD-4/KD-3: composed tax under K-4's ceiling. TD-3 + R8: as ACCEPTANCE.

## Tests to add
- crates/contracts/tests/k5_gate_intensity_contract.rs (~110 LOC): (a) a trivial rename selects the
  always-on tier only (zero expensive-leg rows); (b) a high-cone auth-touching edit selects the full
  stack; (c) the selector is deterministic (same signals -> same tier, no LLM input — TD-3); (d) the
  policy-table hash is frozen (R8); (e) the composed tax fits under a fixture K-4 ceiling (R2 — over
  the ceiling FAILS).

## Files touched
- CREATE: crates/orchestrator/src/gate_intensity.rs (~240 LOC: table + selector)
- CREATE: data/gate_intensity/policy_table.toml (~60 LOC, the R8-frozen table data)
- MODIFY: the auditor gate-dispatch path (~30 LOC: consult the selector)
- MODIFY: .claude/gate-manifest.txt (~1 LOC: the policy-table hash)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (IF +GateIntensitySelected; sequential)
- CREATE: crates/contracts/tests/k5_gate_intensity_contract.rs (~110 LOC)
- CREATE: workflow/reasoning/K-5-decisions.md (~30 LOC)

## Citations / Grounding
- KS-3 report 17: adaptive gating beats gate-everything. KW-4 (collapse-where-safe). [EXTERNAL/DESIGN]
- ELAI ground truth: KG-8 (gate inventory), the risk substrate (B-22/C-30/J-16/F-2/D-15/B-29 — re-grep),
  the leg entrypoints (F-1/F-9/E-30/G-1/I-20/H-26/E-53), K-4 (the ceiling).
- CLAUDE.md: TD-3 (LLM never adjudicates), R8 (frozen gate), GD-4/KD-3 (composed friction bound).

## Open questions / risks
- The trigger thresholds (cone size, novelty, criticality) need calibration — set them OBSERVE-ONLY
  first (the selector logs the would-be tier without enforcing) until the composed tax is measured (GD-4).
- D-15 novelty + B-29 centrality may be PARTIAL/dark — if a signal is unavailable, the policy must
  DEGRADE to a SAFER (more gates) tier, never a cheaper one (fail-safe, not fail-open).
- The frozen-table workflow: the policy table is authored ONCE out-of-band; a plan run that needs to
  change it escalates to the user (R8) — never self-edits.

## Next
This makes the trust spine affordable to keep default-ON: cheap checks always, expensive legs only when
the risk warrants, the whole thing under K-4's ceiling. Together K-4+K-5 are the K-T3 answer to the
gate-tax category-killer.

End of K-5.
