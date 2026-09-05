ELAI FULL-BACKLOG DRAIN — MASTER PLAN (synthesis of the 2026-07-16 6-explorer sweep)

GOAL
Drain the entire ELAI plan corpus (B..Q, ~546 units) systematically in a principled dependency/gate order, WITHOUT letting an old plan revert newer already-shipped work. Replace the walk's capped-40, stale-snapshot-driven frontier with a full-corpus, supersession-aware, delivery-grounded engine; and make the "core" dashboard and the orchestrator update each other LIVE.

Inputs (all under workflow/reasoning/backlog-drain/): cards-BCD.md, cards-EFGH.md, cards-IJKL.md, cards-MNOPQ.md (per-unit state+supersession cards), selector-redesign.md (the queue engine), core-sync-design.md (the bidirectional sync). This doc synthesizes them.

CURRENT STATE (grounded via elai.map symbol+caller greps + gate artifacts, NOT the stale summaries)
- ~546 units: ~185 wired, ~112 partial, ~227 dark (+1 refuted, ~20 uncertain). ~340 units of real remaining work.
- B-D: 132 (~88w / 26p / 14d). E-H: 133 (~52w / 55p / 22d; E is the worst, ~3w/30p/16d). I-L: 217 (~45w / 30p / 130d; J=112 is a "paper letter", only ~9 built; L ~0% built). M-Q: 64 (0 wired, 61 dark, Q-10 refuted, P-25 partial) — the entirely-unshipped frontier.
- The per-letter SUMMARY files are ~2 months stale (mark shipped units "PLANNED"). The walk's status snapshot is 28h stale because the Windows "ELAI Status Watcher" scheduled task is DISABLED (the only feeder trigger); 21 dispositions are stranded WSL->F.

WHY THE CURRENT WALK CANNOT DO THIS (root causes, file:line)
1. Queue = punch head + SUMMARY[:40] (SUMMARY_POOL_MAX=40, plan_selector.py:109); the quorum only ever sees 5 (SELECT_BALLOT_MAX=5, :83). Most of the corpus never enters the queue.
2. The age gate at plan_selector.py:629 returns None and HALTS all selection before build_pool runs (AUTOSEED_MAX_SNAPSHOT_AGE_MIN=180, :56) -> a stale punch kills full-corpus refill even though card/summary enumeration needs no freshness.
3. No supersession-awareness -> risk of building a dead (fully-superseded) unit or reverting newer shipped work.
4. Status is one-way + not delivery-aware (bars come from the frozen 2026-07-02 audit summary).

THE ARCHITECTURE (4 pillars)

PILLAR 1 — the shared plan-index (the spine both sides read/write)
One row per unit: {id, does, state, supersedes[], superseded_by[], residual, key_files[], respect[]}.
- State source of truth: the walk's gate-backed land artifacts — plan-status/{id}.json (DONE + commit_sha + mandatory gate_evidence_ref) + the disposition ledger (drained->wired, escalated/blocked->partial). Zero compute, written every land, strictly fresher than the summaries or I-39.
- Seeded from the 4 cards-*.md just produced (merge -> plan-index.json; run the verify-then-skip pass on the superseded-dark cluster during the merge).
- Lives: WSL conductor/opus_walk/plan-index.json (local) + F: workflow/status/plan-index.json (canonical), kept fresh by the land-write -> feeder-sync -> snapshot -> selector-read pump. Implementation: extend the proven disposition-ledger row with the missing fields (does/supersedes/superseded_by/residual/files/respect). NOT the SSB (append-only, no per-unit key), NOT a new DB table.

PILLAR 2 — corpus_drain.py (the full-corpus ordered engine; NEW module)
- STAGE A classify each unit: DONE | SKIP_SUPERSEDED | BUILD_RESIDUAL | BUILD | RECON.
  SKIP_SUPERSEDED is fail-closed VERIFY-THEN-SKIP: a dark unit skips ONLY if its superseder is proven shipped (is_plan_done OR card-wired), else it stays BUILD. This guard is load-bearing.
- STAGE B collapse residuals (a partially-superseded unit builds only its residual, framed by the superseder).
- STAGE C topo-sort the order: kill-gates first (GATE M-0 blocks all M behind K's kill-gates) -> proof-spine PS-1..PS-6 -> dependency (foundations before extensions) -> hubs (N-1 is the highest fan-in, ~30 promotions terminate there, must precede N-3..N-12; N-2 mechanically emits the N-ranking, do NOT hand-order past it) -> PSR-1 tiebreak. Watch the dark blocking-chains (J-34, J-32 gate most of the J-100s).
- Deterministic pop replaces the blind 5-ballot vote for refill.

PILLAR 3 — per-plan super-summary injection
At plan-start plan_selector._build_pickup (:584) appends corpus_drain.build_brief(unit): the unit's card + its "respect / don't-revert" set. That brief is the "super-summary given at the start of every plan" — the orchestrator works the residual WITH the newer plans in mind.

PILLAR 4 — bidirectional core <-> orchestrator
- Staleness zero-code fix: RE-ENABLE the Windows "ELAI Status Watcher" scheduled task (currently Disabled) — the only feeder trigger.
- Live land-trigger: opus_land.py::finish() LANDED branch (~:403) touches an F: .land-trigger so the feeder fires on each land (live, not 27h-lag).
- Write-back: walk_state.mark_plan_done / _emit_disposition (:420 / :330) upsert the shared plan-index.
- Read: plan_selector reads the index + the true per-letter tally (elai_status_query --letter --json) at plan-start. Core and orchestrator now share one live surface.

THE DON'T-REVERT DENYLIST (narrow + enumerable — this is the key safety, and it is SMALL)
Newer letters overwhelmingly EXTEND/COMPOSE/ACTIVATE older ones; the true revert-traps are a short list:
- L-16 re-specs conversation.rs::compact_session — any drain touching E-25 / H-49 / H-50 compaction yields to L-16 (highest structural hazard).
- H-43 / H-44 / K-19 supersede the LIVE DGM anti-patterns (darwin_admit_loop.rs byte-hash fitness, trap_tasks.rs keyword-grep) — finishing E-7 / F-11 must NOT restore them.
- H-53 -> E-48 (detect-after citation); H-49 -> H-35 + E-25 (restore/checkpoints); G-4 -> E-24 (pin-once); C-48 -> E-5 (SafeHarbor TrustTier now lives in the Mem0 crate crates/memory).
- N is a giant PROSPECTIVE unifier (N-1 unifies 10 benches, N-5 ~16 routers, N-6 six trust scales, ...): finishing a B..L unit before its N-unifier is SAFE, it just adds a retirement-map entry N later absorbs (track it, don't block on it).
These become hard "respect[]" entries on the affected units' cards + a global guard in build_brief.

THE 6 EDIT POINTS (minimal-change, harness-local; nothing in the frozen gate-manifest)
1. NEW conductor/opus_walk/corpus_drain.py (classify / collapse / topo-sort / build_brief).
2. plan_selector.build_pool (:273) — add the corpus tier (read plan-index, uncapped; SUMMARY stays fallback for any uncarded unit).
3. plan_selector.select_next (:612) — split the :629 age gate so staleness DEMOTES the punch, never halts corpus refill.
4. plan_selector._build_pickup (:584) — inject the brief.
5. opus_supervisor._try_autoseed (:328) — auto-disposition SKIP_SUPERSEDED units free (no paid Opus turn).
6. walk_state.mark_plan_done / _emit_disposition (:420 / :330) upsert the shared index; opus_land.py finish() land-trigger; SKILL.md block 2 one-sentence note.
+ Re-enable the Windows "ELAI Status Watcher" task (zero code).

LOAD-BEARING RISK + TEST
The SKIP_SUPERSEDED auto-disposition guard is the one gate-grade risk (wrongly skipping a not-really-covered dark unit = silent loss). It is fail-closed (skip only if the superseder is proven shipped) and MUST get a unit test before it drives any auto-skip.

BUILD ORDER (build the SYSTEM, dry-run, then drain)
Phase 0: merge the 4 cards-*.md -> plan-index.json; run the verify-then-skip confirmation pass on the superseded-dark cluster (C-54..59, the DGM units, etc.).
Phase 1: build corpus_drain.py + the SKIP_SUPERSEDED guard unit test.
Phase 2: wire the bidirectional sync (re-enable watcher, land-trigger, index upsert, selector read).
Phase 3: wire brief injection + the age-gate split.
Phase 4: DRY-RUN corpus_drain — print the computed drain ORDER + the DONE/SKIP/BUILD_RESIDUAL/BUILD classification for every unit, for owner review. NO building yet.
Phase 5: on owner OK of the order, restart the walk on the new engine; it drains in order with grok-4-5-high workers + the per-plan super-summary briefs, landing + updating the live core each unit.

EFFORT
The SYSTEM (Phases 0-3) is ~6 harness files + one test — a small focused build now that the analysis is done. The DRAIN itself (~340 units) is the long-running walk, but now complete, correctly ordered, and revert-safe.

All teikoku harness changes stay owner-hold uncommitted until the owner says commit.
