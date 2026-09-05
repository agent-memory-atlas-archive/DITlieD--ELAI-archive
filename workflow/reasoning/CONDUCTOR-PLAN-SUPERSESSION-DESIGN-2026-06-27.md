CONDUCTOR PLAN SUPERSESSION + BUILD-ORDER DESIGN
2026-06-27

PURPOSE: define the authoritative data source for cross-plan build-order deps (A)
and plan supersession (B), then specify the minimal conductor changes at the
known seams.

================================================================================
1. INVESTIGATION FINDINGS
================================================================================

1.1 ELAI-FEATURE-MAP.md as a dep/supersession source
-----------------------------------------------------
File: docs/thefinalglance/ELAI-FEATURE-MAP.md (~200KB)
Structure: Section 1 = feature catalog (per-feature entries with
STATUS(plan-id) inline); Section 2 = per-letter ledger summaries; Section 3 =
doctrine.

COMPOSITION EDGES present and dense: every feature entry carries EXTENDS /
composes / ACTIVATES / consumes / GATED-on references naming other plan IDs.
Example (verified, line ~346):
  "EXTENDS I-11; composes H-39/I-7/H-50/D-10/SessionDag"
  "PLANNED(J-83) ... EXTENDS I-11 PARTIAL"
  "PLANNED(J-108) ... GATED on J-107"
Hit count for EXTENDS/composes/ACTIVATES/consumes across the map: 144
occurrences on Section 1 alone.

SUPERSESSION signal: absent as a structured field. The map uses the STATUS
enum (WIRED / DARK(flag) / PARTIAL / PLANNED(plan-id) / DEFERRED /
BENCH-PENDING) and tracks stale numbers via inline "STALE" prose, but there is
NO "SUPERSEDED_BY" or "REPLACES" field in any entry. The only machine-readable
supersession in the whole codebase is STATUS_MAP["SUPERSEDED_BY_BPLAN"] = "DONE"
in backlog_import.py:32 (base-phase -> B-plan transition only).

PARSABILITY: the map is semi-structured prose. The STATUS(plan-id) token is
parseable with a regex `\[(?:PLANNED|DARK|PARTIAL|WIRED[^]]*)\(([A-Z]-\d+[^\)]*)\)\]`.
The EXTENDS/composes chains are parseable with a regex over
`EXTENDS\s+([\w\-/+]+)` and `composes\s+([\w\-/+]+)`. Both are extractable
into (consuming-plan, dep-plan) pairs. Coverage: every feature entry in Section
1 has inline status and most have composition tags. Missing: base-phase features
(described without a plan-letter tag) and a small set of single-sentence
entries.

1.2 Plan-index SUMMARY/INDEX supersession keywords
---------------------------------------------------
Pattern searched: supersedes|superseded|SUPERSEDED|replaces|obsoletes|STALE|
stale numbers across all *SUMMARY*.md and *INDEX*.md under docs/thefinalglance/.

Result: 10 total occurrences across 1 file (ALL-PLANS-MASTER-SUMMARY.md:1).
That file's first line is:
  "SUPERSEDED for steal-evaluation context-load by ELAI-FEATURE-MAP.md; ..."
The remaining 9 hits are "STALE" references to stale schema numbers, not plan
supersession. No index file encodes a plan superseding another plan. Verdict:
plan-index files are narrative summaries, not a supersession registry.

1.3 Pre-requisites coverage in plan files
-----------------------------------------
Pattern: "Pre-requisites:|Prerequisites:" across all .md files under
docs/thefinalglance/.

Result: 1 match -- the PLAN-FOLDER-TEMPLATE.md boilerplate line only.

Confirmed by spot-check:
- B-1_path_unification.md line 6: "Pre-requisites: none."
- I-1_directive_supervisor_enforcement_plane.md: NO Pre-requisites field.
- All B-plan files sorted and checked via glob: zero actual dep declarations
  found (B-1 has "none", no B-N has a non-none dep).

The conductor's extract_deps() (backlog_import.py:161) parses Pre-requisites
lines into [A-Z]-\d+ plan IDs. Because essentially no plan file declares
Pre-requisites (other than "none"), the cross-plan dep graph in the DB is
currently empty for B..K plans. The dep machinery exists and works; the data
simply is not being fed.

1.4 Existing machine-readable supersession artifacts
-----------------------------------------------------
Searched: docs/, workflow/ for *supersession*, *replaces*, *stale-plan*,
*supersedes-map*, *feature-map twin*. Result: none found.
The FEATURE-MAP is explicitly described as "the single context-load for steal-
evaluation sessions; replaces the legacy 12-file read." No machine twin exists.
The only programmatic supersession is the SUPERSEDED_BY_BPLAN entry in
STATUS_MAP, covering only base-phase -> B-plan.

1.5 Conductor execution order (confirmed)
-----------------------------------------
backlog_import.py:582: plans sorted by plan_id (alphabetical string sort:
B-1, B-10, B-11 ... B-9, B-43, C-1 ... J-108). This is ASCII order, not
numeric, so B-10 < B-9 and J-1 < J-2 lexically. Letter order IS correct
(B before C before ... J), but within a letter the numeric rank is not
respected.

conductor_db.py:548: ORDER BY t.priority DESC, t.created_at, t.task_id
Priority values: wire_dark tasks=20, READY seeds=15, others=10. No plan-letter
rank factor. All B-plan and J-plan EXECUTE tasks get the same priority=10,
so within-letter execution devolves to creation order (alphabetical import
order), not numeric plan order.

build_tasks at :606: deps list populated ONLY from plan.deps (the parsed
Pre-requisites field). Since Pre-requisites is universally absent/none, deps
list is empty for all B..K plans. Cross-plan blocking is structurally zero
today.

================================================================================
2. AUTHORITATIVE SOURCE RECOMMENDATION
================================================================================

2.1 For CROSS-PLAN BUILD-ORDER DEPS (A)
----------------------------------------
RECOMMENDATION: Parse the FEATURE-MAP's EXTENDS/composes/ACTIVATES/consumes
edges from Section 1 as the primary dep source, PLUS add a "Depends-on:"
field convention to new plan files.

Rationale:
- The FEATURE-MAP is maintained as the living source of truth per project
  memory and maintenance-contract text at lines 20-27. It already encodes 144+
  composition edges, far more than the zero currently in Pre-requisites.
- Parsing it gives inter-plan edges that exist NOW, without authoring changes.
- Risk: the FEATURE-MAP edges name features-as-text, not plan IDs. The
  extractable edges are of the form "EXTENDS I-11" or "composes H-39/I-7"
  which ARE plan IDs, but also "composes SessionDag" (feature names).
  The plan-ID subset is machine-parseable; the feature-name subset requires
  a name->plan-id lookup (trivially built from Section 1's STATUS(plan-id)
  tags).
- Alternative: "Depends-on:" field in plan files (same parse path as
  Pre-requisites, zero ambiguity). This gives precise edges but requires
  authoring for all 300+ existing plans. Not feasible for the seeded backlog.

HYBRID (recommended): FEATURE-MAP parsing for existing plans (build the dep
graph now from what exists), plus a "Depends-on:" header field honored by
extract_deps() for new plans going forward. This covers both the existing
backlog and future authoring with zero migration cost.

2.2 For SUPERSESSION (B)
-------------------------
RECOMMENDATION: Derive supersession from the letter-rank rule (each later
letter supersedes earlier-letter content on the SAME feature) using the
FEATURE-MAP's STATUS field as the filter, not from an explicit supersession
registry.

Rationale:
- No explicit supersession registry exists and the plan-index files carry
  essentially no supersession keywords beyond the base->B transition.
- The project rule "newer letter supersedes older" is stated as a house rule
  (task prompt context). The FEATURE-MAP enforces this implicitly: when a
  feature appears as WIRED/PARTIAL/PLANNED(J-N), the earlier-letter plan that
  first introduced it is already either DONE (seeds from FINISH files) or
  carries a DARK/PARTIAL status indicating the J-N plan is the live version.
- The STATUS field in the FEATURE-MAP gives the current-plan-id for every
  feature. A plan that has been superseded by a later plan either: (a) is
  already DONE in the seeds (audited PASS, safe to skip), or (b) appears as
  PLANNED/PARTIAL for an OLD plan-id when a NEWER plan-id is the active one.
- The actionable supersession signal is: if the FEATURE-MAP has
  PLANNED(X-M) for a feature AND a later plan Y-N (Y > X or N > M same
  letter) also claims that feature with EXTENDS/ACTIVATION/REPLACES, then
  X-M is superseded by Y-N.

Derivation for supersession: for each feature entry, extract the canonical
plan-id from STATUS(plan-id). If another entry's composition edge points to
that plan-id with ACTIVATES or EXTENDS (which means it absorbs the older
plan's deliverable), the older plan can be force-DONE at import time because
its deliverable is consumed by the newer one.

This covers the J-plan wave (where J-83 EXTENDS I-11, meaning I-11's partial
deliverable is completed by J-83 -- I-11 should not block J-83) rather than
superseding it outright. True supersession (old plan fully replaced) is rarer;
most are EXTENDS-composition, not replacement.

PRAGMATIC BOUNDARY: full automatic supersession inference from the FEATURE-MAP
is complex and fragile (prose parsing). The minimum viable signal is: plans
whose FEATURE-MAP STATUS is WIRED and that have a seed verdict SHIPPED_WIRED
are DONE at import -- already handled. Plans that appear only as DARK(flag) with
no active PLANNED(plan-id) consumer are genuinely superseded or deferred.

================================================================================
3. DERIVATION
================================================================================

Step 1 -- FEATURE-MAP dep extraction (new: feature_map_deps.py script):
  Parse ELAI-FEATURE-MAP.md Section 1.
  For each feature line, extract:
    - canonical_plan_id from STATUS(plan-id) token regex
    - composition_deps from EXTENDS/composes/ACTIVATES/consumes/GATED-on tokens
  Build a dict: plan_id -> set[dep_plan_id]
  Where a dep_plan_id appears in composition_deps AND is a valid [A-Z]-\d+
  identifier (not a feature name), add a dep edge.
  Output: JSON file at conductor/state/feature_map_deps.json

Step 2 -- Plan-rank derivation:
  Letter rank: B=1, C=2, D=3, E=4, F=5, G=6, H=7, I=8, J=9, K=10
  Within-letter rank: numeric suffix (B-1=1, B-10=10, B-43=43)
  plan_rank = letter_rank * 10000 + numeric_suffix
  This gives B-1 rank 10001, J-108 rank 91080, etc.
  plan_rank replaces the flat priority tiers in backlog_import.py:436
  (wire_dark and READY adjustments can be additive on top of plan_rank).

Step 3 -- Superseded-DONE force:
  At import time, for each plan P:
    If FEATURE-MAP STATUS for a feature introduced by P is WIRED
    AND the FINISH seed verdict for P is SHIPPED_WIRED -> already DONE, no change.
    If feature entry shows "DEFERRED" or no active PLANNED(Q-N) consumer:
      Mark P as DONE at import (add "DEFERRED" to STATUS_MAP -> "DONE").
  Additionally: plans already seeded DONE via FINISH files stay DONE regardless.

Step 4 -- Build-order priority encoding:
  Insert plan_rank into the tasks table as the primary priority value.
  ORDER BY (existing): priority DESC, created_at, task_id
  Since higher-numbered letters get HIGHER plan_rank -> LOWER priority DESC ->
  EARLIER plans run first. This is the desired behavior.
  Dep edges from feature_map_deps.json feed the task_deps table at :606.

================================================================================
4. MINIMAL CONDUCTOR CHANGES
================================================================================

Seam 1: backlog_import.py:436 -- base_priority
  Change: replace flat {wire_dark:20, READY:15, other:10} with
    base_priority = letter_rank * 10000 + numeric_suffix
  where letter_rank = ord(plan_letter) - ord('B') + 1 (B=1, C=2, ... K=10)
  and numeric_suffix = int(plan_number_part).
  Wire-dark and READY bonuses become small offsets (+50, +20) on top of rank
  so they influence tie-breaking without inverting letter order.

Seam 2: backlog_import.py:161/606 -- dep extraction + dep-edge build
  New file: conductor/feature_map_deps.py (or inline in backlog_import.py)
  At build_tasks time, after parsing Pre-requisites deps, augment plan.deps
  with edges from the feature_map_deps.json lookup.
  extract_deps() remains unchanged (still parses Pre-requisites); the augment
  happens in build_tasks before the :606 dep-edge loop.

Seam 3: STATUS_MAP in backlog_import.py:31
  Add: "DEFERRED": "DONE"
  This forces feature-map-flagged DEFERRED plans to DONE at import, preventing
  them from blocking the frontier.

Seam 4: models.py / conductor_db.py -- schema
  No new column required. The existing priority INT column carries plan_rank.
  No schema migration needed for existing rows IF the DB is re-seeded.
  If backfilling: UPDATE tasks SET priority = <plan_rank> WHERE plan_id = ?
  for each row -- a one-time backfill script.

Seam 5: conductor_db.py:548 -- ORDER BY
  No change required. ORDER BY priority DESC, created_at, task_id already
  gives the right result once priority encodes plan_rank.

================================================================================
5. COVERAGE GAPS + RISK
================================================================================

5.1 Feature-map dep extraction coverage
  The FEATURE-MAP covers all 300+ B..K plans in Section 1 status entries.
  However, not every plan has explicit EXTENDS/composes edges -- plans that are
  entirely new capability (B-1 path unification, B-2 FSM wiring) have no
  upstream deps beyond the base phases. Coverage estimate: ~60% of plans have
  at least one extractable plan-ID dep from the FEATURE-MAP; the remaining ~40%
  are either foundational (truly no cross-plan dep) or have only feature-name
  deps (not plan-ID deps).
  
  For uncovered plans: default behavior is no cross-plan dep edges and
  plan_rank priority ordering. This is safe -- a plan with no declared deps
  runs when its letter rank allows, which is correct for foundational plans.

5.2 Dependency-backfire risk (CRITICAL)
  The most dangerous failure: a newer plan J-N (high plan_rank = lower priority)
  runs BEFORE the older plan B-M it depends on, because plan_rank puts B-M
  first but a dep edge was MISSED (not in feature_map_deps, not in Pre-
  requisites).
  
  With priority DESC and missing dep edge: B-M gets priority ~10001, J-N gets
  ~91080. ORDER BY priority DESC means B-M (10001) is fetched BEFORE J-N
  (91080) because 91080 > 10001 and DESC means HIGHER priority first -- so
  J-N runs FIRST. Wait, this is the wrong direction.
  
  CORRECTION: ORDER BY priority DESC means higher priority number = runs first.
  If plan_rank is letter_rank * 10000 + number, then J-N has the HIGHEST rank
  (newest = most important). But the house rule is "newer supersedes older"
  meaning NEWER should run AFTER OLDER has built the foundation.
  
  INVERSION REQUIRED: newer plans have LOWER priority so older (foundational)
  plans run first.
  plan_rank_priority = (MAX_LETTER_RANK - letter_rank) * 10000 - numeric_suffix
  OR equivalently, ORDER BY plan_rank ASC and invert the DESC.
  
  With B-plan priority highest and K-plan lowest, the frontier naturally clears
  foundational B/C/D plans before attempting J/K plans. Dep edges then prevent
  the rare case where a J-plan's dep is a later E/F plan (the dep edge gates it
  correctly regardless of priority).
  
  Missing dep edge + wrong run order: if J-83 (EXTENDS I-11) has no dep edge
  but I-11 is PLANNED, J-83 would be gated by priority (J later than I = lower
  priority = runs after I). Priority ordering partially compensates for missing
  dep edges within the "foundational before advanced" pattern.
  
  The genuine gap: plans within the SAME letter that have intra-letter deps
  (J-108 GATED on J-107). Numeric suffix handles this correctly IF numeric sort
  is used (J-107 priority = 9*10000+107 = 90107 > J-108's 90108 -- wait, lower
  number = higher priority = runs first, correct).

5.3 Supersession inference accuracy
  The FEATURE-MAP STALE warnings and STATUS flips are maintained by the
  paper-to-plan and planning-workflow skills. They are human-curated, not
  machine-derived. The "DEFERRED" -> DONE mapping covers explicitly deferred
  plans. Plans that are implicitly superseded (newer plan absorbed the scope
  without the map being updated) remain PLANNED in the conductor and will
  attempt execution -- the auditor's wiring gate catches the conflict at
  verification time.

5.4 Live DB backfill vs re-seed
  Conductor has DONE=524 tasks. Priority backfill is safe (UPDATE only, no
  schema change). Dep-edge addition for already-DONE plans is a no-op (done
  tasks do not block anything). Adding dep edges for PLANNED/READY tasks: safe,
  the frontier re-evaluates dep satisfaction on the next acquire_ready call.
  Re-seed is NOT required; backfill is sufficient.
  
  Backfill script: run backlog_import.py --dry-run to get the plan_rank values,
  then UPDATE tasks SET priority = <plan_rank> WHERE plan_id = ?. Feature-map
  dep edges insert via add_dep calls (conductor_db.py existing mechanism).

================================================================================
6. MIGRATION NOTE
================================================================================

The recommended sequence for the live conductor (DONE=524 backlog):

1. Author conductor/feature_map_deps.py: parse ELAI-FEATURE-MAP.md Section 1,
   emit conductor/state/feature_map_deps.json (plan_id -> [dep_plan_id, ...]).
   Run once, re-run when the FEATURE-MAP is upserted.

2. Patch backlog_import.py:
   - :436 base_priority: encode letter + numeric rank, inverted (B highest)
   - :161/extract_deps: after regex parse, augment with feature_map_deps lookup
   - :31 STATUS_MAP: add "DEFERRED": "DONE"

3. Run backlog_import.py --missing-only against the live DB:
   - New plans (J-plan, K-plan) imported with correct rank + edges.
   - Existing rows NOT rewritten (--missing-only flag).

4. Run the backfill UPDATE for priority on existing rows.

5. Insert missing dep edges via a one-time migration:
   for (task_id, dep_task_id) from the feature_map_deps derivation,
   call conductor_db.add_dep if not already present.

No FSM state reset. No DONE rows disturbed.

================================================================================
7. SINGLE BIGGEST RISK
================================================================================

The dependency-backfire failure for SAME-LETTER intra-plan deps with missing
feature-map edges. Example: J-108 is GATED on J-107; if J-107 is not in the
feature_map_deps output (because the GATED-on keyword is not matched by the
EXTENDS/composes regex), J-108 will attempt execution before J-107 lands. The
conductor's dep-edge mechanism blocks it IF the edge exists; without it,
priority ordering alone keeps J-107 (lower suffix) ahead of J-108 (higher
suffix) -- which is correct. The GATED-on/depends pattern in the FEATURE-MAP
must be added to the extraction regex alongside EXTENDS/composes/ACTIVATES.

================================================================================
8. CITATIONS (file:line)
================================================================================

backlog_import.py:31 -- STATUS_MAP (SUPERSEDED_BY_BPLAN only supersession)
backlog_import.py:161 -- extract_deps() (Pre-requisites parser)
backlog_import.py:436 -- base_priority values (flat tiers)
backlog_import.py:582 -- plans sorted by plan_id (alphabetical, not numeric)
backlog_import.py:606 -- dep-edge build from plan.deps
conductor_db.py:548 -- ORDER BY priority DESC, created_at, task_id
models.py:97 -- priority: int = 0 field on TaskSpec
ELAI-FEATURE-MAP.md:1 -- maintenance contract + STATUS enum definition
ELAI-FEATURE-MAP.md:28 -- STATUS enum: WIRED/DARK/PARTIAL/PLANNED/DEFERRED
ELAI-FEATURE-MAP.md (Section 1, ~144 EXTENDS/composes occurrences grep-verified)
PLAN-FOLDER-TEMPLATE.md:119 -- only Pre-requisites occurrence in plan files
ALL-PLANS-MASTER-SUMMARY.md:1 -- "SUPERSEDED...by ELAI-FEATURE-MAP.md" header
