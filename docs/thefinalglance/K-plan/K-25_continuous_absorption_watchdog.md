# K-25: continuous absorption watchdog — K-8's manual quarterly audit made a standing, change-triggered discipline; the provider-surface snapshot + a deterministic overlap checker

STATUS: PLANNED (CAW wave; extends K-8, never re-authors it; KD-4 made continuous; NO SSB; advisory
cache boundary — never a done-gate; the kill/de-scope output stays a PROPOSAL, never auto-applied)
PROOF_SPINE: none (defensive depreciation discipline — protects the spine's ROI by stopping spend on
provider-absorbed units; user-acked 2026-07-05 as a spine-exempt discipline file).

Objective: the spec-eval 2026-07-05 (workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md §4 L2)
named scaffold absorption the single biggest strategic threat with the weakest machinery: K-8 is a
ONE-SHOT sweep + a pre-gate rule, so between sweeps the corpus is blind to a provider shipping natively
what a PLANNED unit builds. K-25 makes the watch CONTINUOUS: (a) a dated, file-based provider-surface
snapshot the harness can check for staleness in subseconds; (b) a deterministic overlap checker joining
the snapshot against the feature-map's ABSORBED/AT-RISK/DURABLE tags (K-8's output) and emitting a punch
list; (c) a staleness trigger wired into the same conditional-routing lane as K-8's pre-gate, so an
efficiency-class task on a stale snapshot must refresh (WebSearch) BEFORE EXECUTING. Same shape and
boundary as elai_status_query.py: an advisory cache, age-stamped, exit-3-on-stale, that speeds the
"is this unit already native?" question and can NEVER discharge a gate.

Estimated effort: ~1 session (S-M, ~4-6 h). The checker is a stdlib tool; the snapshot is a schema +
first fill; the rule is ~10 lines riding K-8's conditional-routing line.
Pre-requisites: K-8 (the ABSORBED/AT-RISK/DURABLE tags in ELAI-FEATURE-MAP.md + the
provider-absorption-pregate.md rule — K-25 extends both). The elai_status_query.py precedent
(.claude/tools/) for the advisory-cache pattern.
Parallel-safe with: K-9 (the model-refresh sibling; disjoint files), all non-K work.
Source: spec-eval 2026-07-05 ADD-2; K-8 (KD-4); KS-3 report 17 absorption record; the elai-status
Layer-0 precedent (workflow/handoff/elai-status-monitor.md). Doctrine: K-0_INDEX.md.

## Context
Why continuous, not quarterly: the absorption record shows a 6-18mo provider cycle (KS-2 §1.2), but
individual absorptions land unannounced mid-quarter (caching markers, tool-search, batch, structured
outputs all landed between ELAI's own plan waves). A quarterly manual audit catches them AFTER build
hours are spent; a staleness-gated snapshot catches them at the moment an efficiency-class unit is
about to start EXECUTING — the only moment that matters. Mechanism: the snapshot holds one row per
watched provider capability {provider, capability_key, status: NATIVE|PARTIAL|ABSENT, source_url,
verified_date}; the checker joins capability_key against the feature-map unit rows' absorption tags
and overlap keywords; any (PLANNED-or-PARTIAL unit) x (NATIVE capability) pair is a punch-list row.
The LLM (not the tool) does the refresh — WebSearch per row, re-date, re-verdict — because surface
verification needs the web; the tool's job is to make staleness and overlap DETERMINISTIC and cheap.

## Existing state (what this composes)
- K-8: the tags (ABSORBED/AT-RISK/DURABLE upserted into ELAI-FEATURE-MAP.md), the pre-gate rule
  (.claude/rules/provider-absorption-pregate.md), the conditional-routing load line. K-25 EXTENDS
  these; it re-authors none (GD-1).
- .claude/tools/elai_status_query.py — the advisory-cache precedent: age-stamped file, --max-age
  staleness exit code, --json, NEVER a done-gate (wiring-enforcement R8-frozen boundary untouched).
- K-9 quarterly model-refresh cadence — the sibling for the MODEL axis; K-25 is the PROVIDER-SURFACE
  axis. Disjoint; the K-9 cadence run is a natural snapshot-refresh trigger.
- G-10 model-drift canary — the runtime EXOGENOUS-change watcher; K-25 is the plan-time one. Note the
  symmetry in the rule text; share nothing (different layers).

## Reads (mandatory before coding)
1. K-8_absorbable_unit_audit_provider_pregate.md (the tags + rule this extends).
2. .claude/tools/elai_status_query.py (the advisory-cache pattern to mirror — head + arg handling).
3. ELAI-FEATURE-MAP.md efficiency-class unit rows (the join target; re-grep tags at edit, W-7).
4. .claude/rules/conditional-routing.md (the load point K-8 added; K-25 rides the same line).
5. workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md §4-L2/§5-ADD-2 (the why).

## Tasks
1. Snapshot schema + first fill: workflow/status/provider-surface.json — one row per watched
   capability across Anthropic/OpenAI/Google/provider SDKs (start from KS-3 report 17's absorbed set
   + every AT-RISK tag K-8 produced). Every row carries source_url + verified_date (anti-stale; a row
   without both is invalid — fail-closed parse).
2. Checker: .claude/tools/absorption_watch.py (stdlib-only, --json, --max-age-days N default 90).
   Verdicts per overlap: NATIVE_OVERLAP (a PLANNED/PARTIAL unit overlapping a NATIVE row — the punch
   line), AT_RISK_STALE (an AT-RISK tag whose backing row is older than max-age), CLEAN. Exit 0 clean
   / 1 overlaps found / 2 config-or-parse error (fail-closed) / 3 snapshot stale past --max-age-days.
   The unit-to-capability join is keyword/tag-based and declared UNSOUND in --help (advisory, R5-style
   honesty); the LLM adjudicates each punch row.
3. Rule extension: .claude/rules/absorption-watch.md (~15 lines) — on any efficiency-class task, run
   absorption_watch.py FIRST; exit 3 = refresh the snapshot (WebSearch, re-date rows) BEFORE
   EXECUTING; exit 1 = surface the punch rows to the user as a kill/de-scope PROPOSAL (house rule: no
   removal without confirmation). Add the one conditional-routing line (same trigger as K-8's rule).
4. NEGATIVE arms: a snapshot row missing source_url/verified_date fails parse (exit 2); a stale
   snapshot returns exit 3 (never a silent CLEAN); the tool's verdict appearing in any done-gate or
   audit PASS is a violation (advisory boundary — mirror the elai_status_query R8 note verbatim).
5. workflow/reasoning/K-25-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - workflow/status/provider-surface.json (the dated snapshot; schema v1)
  - .claude/tools/absorption_watch.py (the deterministic overlap/staleness checker)
  - .claude/rules/absorption-watch.md (+ its conditional-routing load line)
consumes_symbols:
  - ELAI-FEATURE-MAP.md absorption tags (K-8 output; the join target)
  - .claude/rules/provider-absorption-pregate.md (K-8's rule — extended, not forked)
  - .claude/tools/elai_status_query.py (the advisory-cache pattern; pattern-reuse only)
reachable_from:
  - conditional-routing loads absorption-watch.md on an efficiency-class task; the rule's first
    command is the checker [PRODUCTION harness rule-load]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO auto-applied kill/de-scope (punch rows are PROPOSALS; user confirms — house rule)
  - NO snapshot row without source_url + verified_date (fail-closed parse, exit 2)
  - NO done-gate consumption of the checker verdict (advisory cache boundary; R8 untouched)
  - NO network calls in the checker (stdlib, offline; the WebSearch refresh is the LLM's job)
frontend_pair:
  N/A — harness tool + rule + status file. Justified (W-6).
observable_effect:
  trigger: "an efficiency-class task starts; conditional-routing loads the rule; the checker runs"
  expect_ssb: "none (harness leg; JD-1 evidence)"
  expect_artifact: "on a fixture snapshot with one NATIVE row overlapping one PLANNED unit, the
    checker exits 1 and names the pair; on a snapshot older than --max-age-days it exits 3; on a
    row missing verified_date it exits 2; the punch row reaches the user as a PROPOSAL, and no
    plan unit is removed without explicit confirmation"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-25_continuous_absorption_watchdog.md
test -f workflow/status/provider-surface.json
test -f .claude/tools/absorption_watch.py
test -f .claude/rules/absorption-watch.md
rg -n "absorption-watch" .claude/rules/conditional-routing.md      # the rule is actually loaded
python .claude/tools/absorption_watch.py --json                    # runs; exit 0/1/3 acceptable, 2 = fail
test -f workflow/reasoning/K-25-decisions.md
```

## ACCEPTANCE
- [ ] snapshot schema v1 with every row dated + sourced; fail-closed parse on a bad row.
- [ ] checker exits 0/1/2/3 per the verdict table; --json; stdlib-only; offline.
- [ ] rule loaded via conditional-routing on efficiency-class tasks; exit-3 forces refresh-before-EXECUTING.
- [ ] advisory boundary stated in tool --help + rule (never a done-gate; R8 untouched).
- [ ] punch rows are PROPOSALS; zero auto-removal.
- [ ] No runtime code, no SSB, no new deps.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the rule fires on a live efficiency-class task and the checker's exit code steers the
  session (JD-1 artifact), not "the files exist".
- W-2/W-3/W-4/W-6: N/A (harness leg); justified. W-5: the live rule-load + checker run is the trigger.
- W-7: every snapshot row re-verified (dated WebSearch) at refresh time, never assumed.

## Tests to add
- a fixtures/ snapshot trio (clean / overlap / stale / malformed-row) driven by a harness check that
  asserts the four exit codes. No Rust contract (harness leg).

## Files touched
- CREATE: workflow/status/provider-surface.json (~40 rows at first fill)
- CREATE: .claude/tools/absorption_watch.py (~150 LOC, stdlib)
- CREATE: .claude/rules/absorption-watch.md (~15 LOC)
- MODIFY: .claude/rules/conditional-routing.md (~1 LOC: the load line)
- CREATE: workflow/reasoning/K-25-decisions.md (~30 LOC)

## Citations / Grounding
- spec-eval 2026-07-05 §4 L2 (absorption = biggest strategic threat, weakest machinery) + §5 ADD-2.
- K-8 (the tags + pre-gate this extends); KS-3 report 17 absorption record [EXTERNAL].
- elai_status_query.py / elai-status-monitor handoff (the advisory-cache Layer-0 pattern + the
  R8 "never sign off a changeset" boundary, mirrored verbatim).
- CLAUDE.md: no-removal-without-confirmation; anti-stale-training (dated evidence only).

## Open questions / risks
- Join unsoundness: keyword/tag joins will false-positive (a PARTIAL native surface vs a
  verification-coupled unit). Mitigation: the verdict is advisory + LLM-adjudicated; AT-RISK stays
  the honest default; the checker never auto-tags the map.
- Snapshot rot is the failure mode this exists to catch — which is why staleness is an EXIT CODE,
  not a warning line; the rule makes exit 3 blocking-until-refresh for efficiency-class work only.
- Watch-set creep: cap the snapshot to capabilities that overlap an existing ELAI unit; a general
  provider-news feed is out of scope.

## Next
With K-8 (the sweep) + K-25 (the standing watch) + K-9 (the model cadence), the depreciation layer is
closed: units are tagged once, the tags stay live, and no efficiency-class unit starts on a stale view
of what providers already ship.

End of K-25.
