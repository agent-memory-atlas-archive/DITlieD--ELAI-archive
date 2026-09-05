# J-46 doctrine_debt_rate_measurement

- Track J | lever DOCTRINE-PROPAGATION (D4 turned on the propagation itself) | grounding ELAI-DETERMINISM-DOCTRINE.md D4 (measure the objective not a proxy) + the deep-research vote Q4 (ledger amplifies surrogation; fix = random-sample audit + measurer!=measured + observe-only baseline) + ELAI-FEATURE-MAP upsert seam | nature BUILD (derived metric + baseline sweep + feature-map stamp) | effort M-L | WAVE DPW-4 | depends J-43, J-44, J-45, ELAI-FEATURE-MAP.md
- The propagation MUST NOT be headlined as "the doctrine is now enforced" on faith — that is the exact proxy-for-objective failure D4 names (a checker EXISTING is a proxy; doctrine-conformance is the objective). So the propagation is judged on a MEASURED DELTA: a DOCTRINE-DEBT RATE = (open debt rows / total reviewed decisions), tracked over time, per subsystem. Q4 (deep-research vote) warns the ledger itself AMPLIFIES surrogation (people fill rows minimally, proliferate registered exceptions without remediation), so the metric is not just "count exceptions" — it tracks REMEDIATION VELOCITY, is fed by a random-sample audit (DP-2's panel) performed by a non-author (measurer != measured), and ships OBSERVE-ONLY first to take a baseline before anything gates. A subsystem whose ledger says "all clean" but whose debt rate is implausibly zero is itself SUSPECT — the floor's self-test (D3: a dead evidence path must scream, not shrug).

## Steal/build
1. EXP-DP4 (OBSERVE-ONLY, step 1, IS the baseline): the only-WIRED-first baseline sweep — run J-43's floor + J-44's shape-check + the random-sample substance audit across the WIRED features first (cheaper, judges what actually runs), establishing the baseline doctrine-debt rate per subsystem BEFORE any DD-* class gates. The full B..K sweep runs as a background debt-burn-down, not a blocking pass (the ~213 plans were mostly authored BEFORE the doctrine was written 2026-06-16; a large backlog is a SEQUENCING fact with a promotion plan, not a reason to gate retroactively and stall). Nothing default-ON-blocking before this baseline + the per-class FP rates (J-43 EXP-DP1) are in.
2. THE DOCTRINE-DEBT-RATE METRIC: a derived figure = open doctrine-debt-ledger.toml rows / total reviewed decisions, per subsystem, over time. Computed by .claude/tools/doctrine_debt_rate.py from doctrine-debt-ledger.toml (J-43) + the per-subsystem reviewed-decision count + the DP-2 substance-audit REMEDIATION-VELOCITY (rows closed per window, not just count). Every number recomputable from its source files (GD-2 / G-5 DSR — derived, never recalled). The propagation's success is the DELTA in this rate, not the existence of a checker (D4 on itself).
3. THE ONLY-WIRED-FIRST BASELINE SWEEP across B..K: enumerate the WIRED features (from ELAI-FEATURE-MAP.md status flags), run the floor + shape-check + a sampled substance audit on each, record findings as ledger rows + a per-subsystem baseline rate. Seeds J-45's calibration corpus with real findings. The full sweep (PLANNED/DARK features too) is a background queue.
4. THE ELAI-FEATURE-MAP CONFORMANCE STAMP: each feature in docs/thefinalglance/ELAI-FEATURE-MAP.md gains a doctrine-conformance flag — clean | debt(row-id) | unreviewed — upserted in-place on the SAME FEATURE-MAP UPSERT seam planning-workflow/orchestrate already run (one flag flip, never a narrative). unreviewed is treated as DEBT (D3: absence of a review is not a pass). The stamp makes per-feature doctrine status queryable from the living source of truth.
5. THE IMPLAUSIBLE-ZERO SELF-TEST (D3): doctrine_debt_rate.py FLAGS any subsystem reporting a zero debt rate with a non-trivial reviewed-decision count as SUSPECT (a dead evidence path / a gamed ledger screams, never shrugs) — routed to the DP-2 substance-audit panel for a deep look. A clean rate must be EARNED by an audited sample, not asserted.
EXCLUDED: a leaderboard-style "doctrine score" headline (D4: SAFEDELTA G-6 precedent — ablation-delta not a rank); counting exception rows without remediation velocity (Q4 surrogation trap); a retroactive blocking sweep that stalls all work (observe-only first, backlog gets a promotion plan); trusting an unaudited "all clean" ledger (the implausible-zero self-test exists to catch it); an LLM computing the rate (it is a deterministic count over off-agent files).

## Existing ground truth
(cite the grounding map DP-* IDs; re-grep at edit, W-7)
- docs/thefinalglance/ELAI-FEATURE-MAP.md — the living source of truth (features + status PLANNED/WIRED/DARK + composition edges; the Section 3 doctrine blocks W/GD/HD/TD/VD/RV/CL/ID/JD; the REJECTED ledger). The conformance stamp upserts here; the WIRED-status flags drive the sweep enumeration. (DG-16; the FEATURE-MAP UPSERT is a documented MEMORIZING/orchestrate step.)
- doctrine-debt-ledger.toml (J-43) — the metric's numerator source (open rows + promotion_plan_id + expiry).
- the DP-2 substance-audit panel (J-44) — supplies remediation-velocity + the deep-audit catch-rate the metric tracks; the implausible-zero flag routes BACK to it.
- .claude/tools/maintainability_check.py / bn_status_report.py — the derived-metric tool shape (stdlib, --json, recomputable-from-source). doctrine_debt_rate.py is a sibling. (DG-2, shared with J-43.)
- SAFEDELTA G-6 / E-9 — the "measure the ablation delta, not a leaderboard rank" precedent the rate honors (D4). (DG-17.)

## Integration
A derived-metric tool + a baseline sweep + a FEATURE-MAP stamp, all reading off-agent files (ledger, feature-map, audit-log). No new gate; the metric is a measurement, and the propagation is JUDGED on its delta. Harness leg; done-gate JD-1 (a real sweep produced a baseline rate file + real feature-map stamps + a real implausible-zero flag fired on a seeded gamed ledger). Honest residue (DP design section 9 + V4): the baseline backlog will be large (pre-doctrine plans); the rate is only as honest as the substance-audit sampling behind it (a low rate from thin sampling is the surrogation trap, not a win); per-subsystem reviewed-decision counts are themselves an estimate that must be defined consistently or the rate is not comparable across subsystems.

## WIRING_CONTRACT
produces_symbols: [doctrine_debt_rate.py, the per-subsystem baseline-rate artifact, the ELAI-FEATURE-MAP doctrine-conformance stamp (clean|debt(id)|unreviewed), the implausible-zero self-test flag]
consumes_symbols: [doctrine-debt-ledger.toml (J-43), the DP-2 substance-audit remediation-velocity (J-44), ELAI-FEATURE-MAP.md WIRED status flags + the UPSERT seam, the J-45 calibration corpus (sink)]
reachable_from: [the only-WIRED-first baseline sweep over real WIRED features; the doctrine_debt_rate.py invocation over the ledger; the FEATURE-MAP UPSERT step in MEMORIZING/orchestrate]
stub_ban: [no headline "doctrine score" rank (delta-only, D4); no exception-count without remediation-velocity; no unaudited "all clean" pass (implausible-zero flags it); unreviewed status = debt, never silently clean; every number recomputable from its source file (GD-2)]
observable_effect (JD-1, off-agent): the baseline sweep over WIRED features produces a per-subsystem doctrine-debt-rate file recomputable from doctrine-debt-ledger.toml + the reviewed counts; a real feature flips its FEATURE-MAP stamp to debt(row-id) on the upsert seam (grep-provable); a seeded gamed ledger (zero rows, high reviewed count) trips the implausible-zero flag and routes to the substance-audit panel; the negative arm — an unreviewed feature — reads as DEBT, never clean.
SSB: net 0 new action_type (harness/doc leg; the rate file, the ledger, and the FEATURE-MAP stamp are off-agent artifacts, JD-1 evidence; FEATURE-MAP is a doc, not SSB). If J-43 ships the OPTIONAL DoctrineDebtEvent, the rate can derive from those rows instead; default net 0.

## Grounding
- ELAI-DETERMINISM-DOCTRINE.md D4 (measure the objective not a proxy) + the "absence of a review is not a pass" (D3) — VERIFIED (read this session). The metric's whole posture (judge the delta, treat unreviewed as debt).
- the deep-research vote Q4 (DP design addendum c): a tracked exception rate + ledger AMPLIFIES surrogation unless paired with a random-sample audit + measurer!=measured + remediation-velocity + observe-only baseline + held-out un-gamed subsets. VERIFIED-as-report-finding (governance literature); the mandate for remediation-velocity + the implausible-zero self-test. PROVISIONAL on THIS domain.
- SAFEDELTA G-6 / E-9 (ablation-delta over leaderboard rank) — VERIFIED-in-repo (memory + doctrine D4 example). The "delta not rank" precedent.
- ELAI-FEATURE-MAP.md as the living source of truth with an UPSERT seam — VERIFIED (memory project_feature_map_canonical + the planning-workflow MEMORIZING FEATURE-MAP UPSERT step this session).

## COMPOSES-WITH
- J-43 (DP-1) — reads doctrine-debt-ledger.toml (the numerator); the OPTIONAL DoctrineDebtEvent is an alternate source.
- J-44 (DP-2) — consumes the substance-audit remediation-velocity; routes implausible-zero flags back to its panel.
- J-45 (DP-3) — the baseline sweep's real findings SEED the calibration corpus.
- ELAI-FEATURE-MAP.md + the UPSERT seam (planning-workflow MEMORIZING / orchestrate phase-end) — the conformance-stamp host.
- SAFEDELTA G-6 — the delta-not-rank measurement precedent.
- DP-5 (J-47) — the program-level A/B/C falsifier consumes this debt-rate as one of its leak/quality measures.
- ELAI-DETERMINISM-DOCTRINE.md D4 — the canonical measure-don't-assert application, turned on the propagation itself.

## Open questions / risks
- "Total reviewed decisions" denominator definition: per-feature decision counts must be defined consistently (what counts as a "decision") or the rate is not comparable across subsystems or over time. Pin the definition in the metric tool, not per-sweep.
- Backlog framing: the pre-doctrine backlog will dominate the first baseline; report it as a sequenced burn-down with promotion plans, NOT as a damning headline number (the rate's DELTA over time is the signal, not its absolute first value).
- Stamp staleness: a feature's conformance stamp can drift from reality as the feature changes; tie a re-stamp to the FEATURE-MAP UPSERT seam (already fired on every shipped phase) so it cannot silently age. A stamp older than the feature's last touch = unreviewed = debt.
- The implausible-zero threshold is itself a tuned constant (what reviewed-count makes a zero rate suspect) — a named constant in the tool, observed before trusting.

## Next
J-47 builds the program-level A/B/C falsifier (does this whole hybrid propagation beat doing-nothing AND a comprehensive scorer) + the per-axis doctrine-family discipline (author a new axis-doctrine ONLY on measured pain) — step 1 of the entire DP program.
