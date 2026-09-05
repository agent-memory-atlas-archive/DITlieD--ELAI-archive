ELAI STATUS DASHBOARD - IDEA PLAN (personal harness tool, NOT an ELAI feature)

author session 2026-07-04. this file pins the WHAT + the data pipeline.
2026-07-05: scope, delivery, refresh and design DIRECTION locked by grill, see section 12.
data-source LOCKED: HYBRID (user, 2026-07-04). delivery LOCKED: a TEIKOKU SIGNAL tab (user, 2026-07-05).
design direction LOCKED: lain living-mind (section 12); 3 divergent html mockups precede implementation.

================================================================================
1. WHY THIS EXISTS (the chicken-egg)
================================================================================
The user is building ELAI. ELAI will eventually SHOW its own wiring state (canvas H-10 CODE mode +
J-31 SYSTEM mode - see workflow/reasoning/canvas-how-it-works-and-looks.md). But the harness used to
BUILD ELAI is not ELAI, so there is no in-product view yet. The user needs an EXTERNAL, personal tool
NOW to eyeball "how close is ELAI to done" without spawning agents / burning tokens to re-derive status.

This is HARNESS TOOLING for the user. It lives in the workflow root (.claude/tools/ + a view), NOT in
.elai_cc. It is the scrappy external cousin of J-31; do NOT rebuild J-31 inside ELAI to satisfy this.
When ELAI's own J-31 SYSTEM mode ships, this tool is retired.

================================================================================
2. THE CORE REFRAME - BURNDOWN, NOT HEATMAP (the load-bearing decision)
================================================================================
The naive version (a 296-cell grid colored WIRED/PARTIAL/DARK) MISLEADS. It would read "~42-58% wired"
and imply the user is far from done. That is false. Per workflow/reasoning/plan-audit-2026-07-02/
_MASTER-SYNTHESIS.md, most of the "unwired" backlog is DEFERRED BY DESIGN, not unfinished:
  - registered-dark done right (H-5, I-34/36/37/47/50, C-31/45) - promotion plans named, intentional.
  - GPU/bench-pending (G-6 SAFEDELTA, I-16/25/28/29/30/31) - blocked on RunPod 2x3090, not on the user.
  - user-deferred (C's 8 external harnesses) - confirmed no drift.
  - verification debt (D's 11 UNVERIFIED-no-journal) - AUDIT gaps, not code gaps.

The GENUINE remaining work is small and already enumerated: the _MASTER-SYNTHESIS P0 punch list is ~7
items, most one-liners (I-33 zero-caller; H-10 PagerankPanel orphan; B-4 stale doc-comment; C-15 entropy
gate regression; H Track-3 gates built-but-not-intercepting). "How close am I" = THAT list, not a %.

DECISION: the tool is a PUNCH-LIST BURNDOWN. Each row = one real gap, tied to the deterministic checker
that AUTO-FLIPS it green. That is what delivers the two user needs at once: see status visually AND
without asking an LLM.

================================================================================
3. THE METRIC THAT MATTERS (lead with this; treat % as mood-lighting)
================================================================================
TRUSTWORTHY (checker-grounded, cannot go stale):
  open-count = (open punch-list items) + (live DARK symbols NOT in wire-dark-allowlist.toml)
SOFT (do not lead with it): "% of plans WIRED". The denominator DRIFTS - the user keeps authoring new
plan letters (B..M now, more coming), so % is measured against an expanding spec. Show it as context,
never as the headline.

================================================================================
4. DATA PIPELINE - HYBRID (LOCKED)
================================================================================
DENOMINATOR ("what ELAI should be" + the punch list) = read existing artifacts (instant, no build):
  - the B..I (and later J..M) plan corpus under docs/thefinalglance/*-plan/
  - the latest punch list: workflow/reasoning/plan-audit-2026-07-02/_MASTER-SYNTHESIS.md
  - per-letter roll-up: workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md
  - bn_status_report.py (already parses B-plan status notes: SHIPPED/IN-FLIGHT/PLANNED/MISSING)

NUMERATOR ("what is ACTUALLY wired right now") = run the deterministic checkers live (ground truth,
no LLM, self-updating):
  - python .claude/tools/production_caller_check.py --files <changed .rs> --json   (DARK pub fns, .rs)
  - python .claude/tools/python_wire_check.py --files <changed .py> --json          (DARK defs, .py)
  - python .claude/tools/wiring_contract_check.py --bn-file <plan> [--json]         (per-plan legs)
  - python .claude/tools/gate_freeze_check.py                                       (gate frozen ok)
  - action_type_lock count / SCHEMA_VERSION from ssb.rs                             (build-green proxy)
  - codegraph MCP (codegraph-elai) or elai_graph.json                              (dead-code topology)

SUBTRACT (so intentional-dark does not render as red):
  - .elai_cc/wire-dark-allowlist.toml rows + in-code WIRE-DARK[<plan-id>] markers.

RUNTIME "verified" leg (optional, strongest evidence): the SSB journal (.elai_cc ssb.db) - an
observable_effect row proves the code actually RAN. Only present after a real `elai run` has journaled;
absent for most of the D set today (the UNVERIFIED-no-journal state). Represent it as its own tier, do
not collapse it to a green check.

The feeder script emits ONE status.json; the view is a pure projection of that json. Refresh = re-run
the feeder. No agents anywhere in the loop.

================================================================================
5. THE STALENESS TRAP (do not repeat the thing that already burned a month)
================================================================================
workflow/handoff/wire-remediation.md is a chronicle of the plan-status PROSE verdicts going stale and
contradicting each other SIX times in one month (60->61->94->89 WIRED; a D tally flipping 0<->16 off a
single tool grep failure; ~14M subagent tokens for one 213-plan sweep). ROOT CAUSE: those verdicts are
hand/agent-written prose. RULE FOR THIS TOOL: the NUMERATOR must come from the checkers + ssb.db, NEVER
from the prose fragments. The artifacts are fine for the DENOMINATOR (the roadmap of what to build); they
are NOT trustworthy for "is it wired now". Cross this wire and you rebuild the treadmill.

================================================================================
6. HONEST CAVEATS (must be visible in the view, per wiring-enforcement.md residual-risk)
================================================================================
  - STATIC LEGS ARE UNSOUND. No sound static reachability for Rust with tokio + dyn Trait; false
    positives on trait-object / IPC-string / event-bus dispatch. So the view MUST split CONFIRMED-DEAD
    (runtime/SSB/coverage evidence) from SUSPECTED-DEAD (topology-only). A single red light WILL lie and
    make the user "fix" things that are actually reachable via a dynamic seam.
  - RENAME / STALE-MAP DRIFT. elai.map goes stale within hours of live edits; renamed symbols read as
    0-hit false-negatives (documented all over wire-remediation.md). Prefer codegraph (live, file-watched)
    over elai.map for the topology leg; regen elai.map before trusting it.
  - VERIFIED RUNG NEEDS A RUN. The strongest tier only lights after a journaled `elai run`. Show
    "static-wired, runtime-unproven" as a distinct state - it is useful (it names exactly what a smoke run
    would confirm), not a failure.
  - DENOMINATOR DRIFTS (section 3). New plan letters keep landing; % complete is soft by construction.

================================================================================
7. SEGMENTATION (the buckets the burndown groups by, so deferred work stops scaring the user)
================================================================================
  OPEN            genuine unbuilt gap or defect (the real TODO; the P0/P1 punch items).
  REGISTERED-DARK intentional, has WIRE-DARK marker + allowlist row + named promotion plan. DONE.
  GPU/BENCH       code wired, needs RunPod 2x3090 / a benchmark RUN. Blocked on hardware, not the user.
  USER-DEFERRED   explicitly parked (C's external harnesses). Not counted against completion.
  VERIFY-DEBT     built + static-wired but runtime leg UNVERIFIED-no-journal. Needs a run, not code.
Only OPEN counts against "am I done". The others are shown but do not inflate the open-count headline.

================================================================================
8. PLACEMENT + STACK
================================================================================
  - feeder script: .claude/tools/ (python stdlib is fine - matches every existing checker; it just shells
    the checkers + parses the corpus + writes status.json).
  - any JS/TS leg (a served page, a build step): BUN, never node/npm/npx (.claude/rules/bun-not-node.md).
  - workflow root only. NOTHING written into .elai_cc for this tool.

================================================================================
9. PRIOR ART IN-REPO (reuse, do not reinvent)
================================================================================
  - bn_status_report.py - already produces a SHIPPED/IN-FLIGHT/PLANNED/MISSING punch list from B-plans.
    The feeder can wrap/extend it rather than re-parse plans from scratch.
  - .claude/tools/visual/ (bun + CDP capture, ui-diff, ax-scaffold) - if the design session wants
    screenshot-diff regression on the dashboard itself.
  - .claude/mcp/canvas/ (port 7323, dependency-cruiser depGraph) - a DIFFERENT harness tool (TS/JS graph
    only); not the wiring-state view, but a reference for "local graph served on a port".
  - graph_reconcile.py - unions codegraph + elai.map with confidence tags; use it when the topology leg
    must be TRUSTED (it flags where the two graphs disagree = exactly the false-negative class above).

================================================================================
10. LEFT OPEN FOR THE DESIGN SESSION (the LOOK - do not decide it here)
================================================================================
Hand this brief to /cook (design discovery) then .claude/rulebook/design-taste.md. Open questions for
that session, seeded but NOT answered:
  - primary layout: a single headline open-count + a burndown list? a segmented bar? a per-letter B..M
    strip? a real graph (dead-code islands like H-10) or a table? (the reframe argues AGAINST a 296-cell
    heatmap as the primary; a heatmap can be a secondary drill-down.)
  - how CONFIRMED-DEAD vs SUSPECTED-DEAD reads visually (the honest-uncertainty requirement, section 6).
  - how the five segments (section 7) are visually separated so OPEN dominates and deferred recedes.
  - refresh affordance: a button that re-runs the feeder vs a file-watch vs manual re-run.
  - ELAI house look (teal-on-charcoal, JetBrains Mono, outlined-buttons-only, information-density) per
    dashboardRules.md / design-taste.md, or a deliberately-distinct personal-tool aesthetic.
The design session OWNS these. This file owns the data contract (status.json shape) they render.

================================================================================
11. USER DECISIONS (verbatim, do not re-litigate)
================================================================================
  - 2026-07-04: this is a tool FOR THE USER to check ELAI status, NOT a feature to implement in ELAI.
  - 2026-07-04: data source = HYBRID (artifacts for the roadmap/denominator, live checkers for the
    wired-now/numerator).
  - 2026-07-04: delivery form (hosted Artifact vs local bun-served page vs terminal) = DEFERRED to a
    follow-up session that decides the look. [RESOLVED 2026-07-05, section 12: TEIKOKU tab]

================================================================================
12. GRILL RESULT 2026-07-05 (scope + delivery + refresh + design direction LOCKED)
================================================================================
DELIVERY: a new tab inside TEIKOKU SIGNAL (like the canvas tab), NOT a standalone bun page.
  View code lives in ~/teikoku/signal; the feeder stays here in .claude/tools/ (amends section 8:
  the view is the one piece outside the workflow root). Retired when ELAI's own J-31 ships.

SECTIONS (5, all user-selected):
  1 punch-list burndown  headline = open-count (locked reframe, section 2).
  2 benchmarks board     per-bench: harness exists / ever RUN / GPU-blocked. A number renders ONLY
                         from a real result artifact; otherwise NOT RUN, never a fake value.
  3 ui-rework backlog    the full ELAI dashboard redo (user 2026-07-05: unhappy with current UI,
                         entire rework queued after backend done). Hand-seeded keep/redo list per
                         view, seeded with the user at the design session. This IS the todo record.
  4 dashboard health     auto-run svelte_mount_reachability.py + tauri_ipc_symmetry_check.py over
                         the CURRENT ELAI dashboard (orphan components, dead IPC wires).
  5 usage meter          reuse conductor GET /usage (accounts[] percent rows + per-lane tokens).
                         Tokens and percent-of-plan, never dollars (MAX plan, user 2026-07-05).

PIPELINE: elai_status_feeder.py (new, .claude/tools/, stdlib) shells the existing checkers +
  parses the plan corpus (dynamic glob docs/thefinalglance/*-plan, so N+ letters auto-appear) +
  subtracts allowlist/WIRE-DARK + reads bench artifacts + SSB last-run stamp, writes ONE
  workflow/status/elai-status.json. A Windows logon-task watcher polls the repo git state every
  ~2min: changed -> re-run feeder; plus a daily forced run. Conductor gets read-only GET
  /elai-status serving /mnt/f/...(the json). The tab polls /elai-status ~10s + /usage ~30s.
  "Almost real time" = change-triggered, since the numbers only move when the repo moves.

HONESTY RULES: a crashed/missing checker renders that section as ERROR, never green; every
  section carries its own "as of" stamp; CONFIRMED-DEAD vs SUSPECTED-DEAD stays split (section 6);
  walk activity NOT duplicated here (the canvas tab owns the live conductor walk).

DESIGN DIRECTION (locked 2026-07-05; mockups before implementation):
  A living AI mind: a crazy 3Dish particle-brain centerpiece (VAULT-HUD reference) where every
  node is a REAL plan/feature unit - lit/connected = wired, dark = not - and the mind visibly
  COMPLETES as ELAI completes. Palette + vibe = serial experiments lain (cold band, wired-blue
  near-black, glitch teal, violet; the owner-approved teikoku-brain lain gradient is the house
  precedent). SIGNAL structural discipline holds (contrast gates, mono for data, honest states,
  anti-slop bans) with OWNER-MOTIVATED exceptions to B-05/B-06 for this surface only (the
  animated mind + the lain cold band). 3 VERY different standalone .html mockups by an opus
  agent land in workflow/design/mockup/ for the user to react to; no implementation before the
  user picks. Build order after pick: feeder -> watcher -> conductor route -> bare tab -> skin.
