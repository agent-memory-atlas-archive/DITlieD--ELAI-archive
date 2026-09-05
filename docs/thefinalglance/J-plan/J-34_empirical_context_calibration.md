# J-34: empirical context calibration — measured per-model fill-degradation curves + positional placement policy (kill the folklore thresholds)

STATUS: PLANNED (WAVE CC-1)

- Track J | lever PROVABLE+CHEAPER | grounding STRONG (every substrate + absence claim code-verified 2026-06-13) | nature BUILD-on-shipped (calibration layer over H-50/3E/E-49) | effort M-L | WAVE CC-1 | Tier-2 dep: J-28 (journal-regression leg) | invalidation composes G-10

## Premise

ELAI's context-lifecycle design already matches published best practice (stable cache prefix,
append-only, mask-don't-remove, regime split by lane — H-50/H-6/H-2). What NOBODY ships — not
ELAI, not the field — is calibration: the thresholds and layouts are folklore constants. The
public state of the art is literally a Reddit thread arguing whether degradation starts at 42%
or 67% of the window. ELAI's own numbers are the same kind: CLOUD_TRIM_FRACTION = 0.85 hardcoded
(context_policy.rs:70), the local window from an env var, slot ORDER fixed by convention. The
lost-in-the-middle U-curve and context rot are real, MEASURABLE, per-model phenomena — so
measure them, per model, on ELAI's own fixtures and journal, and let the shipped machinery
consume curves instead of constants. This is the doctrine ELAI applies everywhere else (HD-1 /
ID-5 / "believe the number") applied to its own context manager: turn "compact at 85%" into
"compact where THIS model's measured verified-outcome curve knees", and "rules first, task last"
into a per-model placement policy derived from a probe, not a vibe. Genuinely unplanned
(GD-1 grep 2026-06-13: E-49 has a fixed 5-position pass/fail context_rot_probe; I-16's
degradation curve is KV-pruning accuracy; E-32 FITM is a logit-level positional prior,
local-only; I-6 is a structural anchor slot — composers all, no unit measures per-model fill
thresholds or drives slot ordering from data).

## Steal/build (4 legs)

1. FILL-DEGRADATION CURVE (`elai bench context-fill`): run a fixed fixture task set at stepped
   context-fill levels (10%..95%), padding with DETERMINISTIC real-repo content (never lorem —
   padding must look like production context; seeded selection under ELAI_SEED), scoring
   verified-OUTCOME rate (W-1-grade: compile/test/gate verdicts, never LLM-judged — TD-3). Output
   per (model, lane): a fill->outcome curve + its knee, written to a calibration artifact +
   metrics TOML, byte-recomputable from the journal (GD-2). The Reddit "42%" becomes a measured,
   CI-carrying, per-model number.
2. CONSUME THE CURVE: ModelCapabilityProfile (context_budget.rs:292) gains
   fill_degradation_curve + calibrated_trim_fraction (Option fields, absent = today's behavior);
   context_policy.rs::trim_threshold gains a per-model lookup that PREFERS the measured value
   over CLOUD_TRIM_FRACTION/the env constant when present (clean extension — the current fn
   takes only the policy enum; recalibrate_budget at context_budget.rs:849 is the existing
   hook). H-50's regime split is unchanged; its thresholds stop being folklore.
3. POSITIONAL PLACEMENT POLICY: extend E-49's context_rot_probe primitive (fixed 12K synthetic,
   5 depths, pass/fail) into a positional-value PROBE: constraint-recall fixtures placed at
   stepped depths d, deterministic honored/violated scoring, output a per-model positional-value
   curve -> ModelCapabilityProfile.positional_value_curve. The 3E slot allocator gains a
   PLACEMENT policy: slot ORDER chosen by (slot recall-sensitivity x measured positional value)
   — high-sensitivity slots (active constraints, task spec, rules deltas) pinned to the measured
   high-recall edges, bulk recoverable content (CodeContext, SsbEntries) in the measured trough.
   CACHE-SAFE BY CONSTRUCTION (HD-2): the layout is fixed PER MODEL at calibration time, never
   per-call — the byte-stable prefix ordering only changes when the calibration artifact
   changes, which is a logged, journaled event, and the bp.valid assert still guards it.
   I-6's AnchorPane stays the last-position special case this generalizes; E-32 FITM remains the
   orthogonal logit-level mitigation on the local lane (placement works on BOTH lanes, including
   cloud where logits are unreachable).
4. INVALIDATION + Tier-2: calibration artifacts are keyed by model identity; a G-10
   model-drift/identity change marks them STALE (recalibrate, never silently reuse — the same
   newest-wins discipline as the schema stamps). Tier-2 (gated on J-28 UsageTurnEvent shipping):
   the journal-regression leg — regress verified-outcome vs fill% from REAL production turns per
   (model, task-class), refining the bench curve with field data; until J-28's
   (model, task, session)-keyed rows exist this leg has no data (verified: no current SSB row
   carries those keys) and the padded-fixture bench stands alone.

## Existing ground truth (code-verified 2026-06-13; re-grep at edit, W-7)

- context_policy.rs SHIPPED (H-50, PARTIAL/uncommitted): CLOUD_TRIM_FRACTION = 0.85 (line 70),
  local from ELAI_LOCAL_MAX_CONTEXT, trim_threshold(policy, window) at :139 — NO per-model hook
  today. ModelCapabilityProfile at context_budget.rs:292: capability descriptor only, NO
  positional/degradation fields; recalibrate_budget at :849. 3E allocator + slot_weights WIRED.
- E-49 context_rot_probe: fixed 12K synthetic, 5 positions (0.0..1.0), 70% pass target — the
  probe PRIMITIVE this extends (pass/fail canary != calibration curve).
- E-32 FITM SHIPPED-DARK: FoundInTheMiddleCalibrator LogitsProcessor (server.py:1564,
  ELAI_FITM_ENABLED, >8K prompts) + FitmCalibrationApplied SSB (ssb.rs:6391) + bench
  (benches/context_window/run_fitm_eval.py) — logit-level, sidecar-only; composes, no overlap.
- J-28 PLANNED-ONLY (UsageTurnEvent absent from elai.map) — hence the bench leg is Tier-1 and
  self-contained. G-10 PLANNED (model_drift.rs substrate shipped); ModelIdentity::identity_key
  is the calibration cache key surface. I-27/E-49 scope = outcomes vs COMPUTE/$ — no overlap
  with outcomes vs FILL.

## REVISED scope (W-10 tiered)

Tier-1: legs 1+2 for the LOCAL lane + cloud-fixture leg under a small fixed budget (the cloud
curve costs real tokens — run it on the cheap workhorse model under an explicit E-39 cap, never
opus-class; record cost in the TOML). Leg 3 ships behind the same calibration artifact with the
allocator consuming it only when present.
Tier-2: J-28 journal regression; per-task-class curves (needs row volume); G-10-triggered
auto-recalibration (until G-10 ships: manual `elai bench context-fill --recalibrate` on model
change).

## Integration

- NEW crates/orchestrator/src/context_calibration.rs (~250 LOC: curve types, knee detection,
  artifact store keyed by model identity) + crates/cli bench arm `context-fill` (mirrors the
  seccodeplt/qdelta runner shape, args_bench.rs).
- Edits: context_policy.rs (per-model lookup), context_budget.rs (profile fields + placement
  policy in slot assembly), E-49 probe extension (depth-stepped scoring).
- SSB: +ContextCalibrationEvent (Pattern-2 single-variant; Kind: CurveMeasured /
  ThresholdApplied / PlacementApplied / CalibrationInvalidated). baseline+1 from LIVE 241/550
  (re-grep at edit, W-9; full 6-part discipline). ThresholdApplied/PlacementApplied rows make
  every consumption journal-visible (a threshold the policy used but never logged = R1 FAIL).
- frontend_pair: the measured curve + knee + active thresholds rendered on the
  ContextStatePanel/cost-cockpit family (the "your model degrades HERE" tile — the
  believe-the-number instrument for context).

## observable_effect (done-gate, W-1)

`elai bench context-fill` on the local lane emits CurveMeasured rows whose curve recomputes
byte-identically from the journal under ELAI_SEED (GD-2) and writes the calibration artifact; a
subsequent real session shows ThresholdApplied with the MEASURED trim fraction (not 0.85) in the
H-50 policy path, and the panel renders curve + knee. With leg 3 present, the assembled prompt's
slot order provably follows the placement policy (journaled request bytes) AND the HD-2
byte-stability assert still passes across turns (placement is per-model-static). NEGATIVE (R2):
a model with no calibration artifact uses today's constants AND logs that fallback (no silent
pseudo-calibration); deleting the artifact mid-session does not change layout until the logged
recalibration boundary (no per-call layout churn — the cache-bust guard proves it); a broken arm
scoring the curve via LLM-judge instead of gate verdicts FAILS the contract test (TD-3); padding
drawn from lorem/random instead of seeded repo content FAILS the fixture audit.

## Riders (recorded, not in-scope)

- Publishing the measured curves (per-model degradation table) as a docs/strategy artifact =
  K-1 claims-policy material once numbers exist; recorded.
- Needle-in-haystack style retrieval probes = explicitly NOT the metric (retrieval != task
  outcome); the probe scores constraint-HONORING on real task shapes.
- A live in-session rot meter (mid-session probe calls) = DECLINED for cost; the ledgered
  fill% vs the calibrated knee on the panel is the live instrument.
- Cross-hardware variance on local curves = recorded honestly in the artifact (same-box
  same-quant validity scope, GD-5).
