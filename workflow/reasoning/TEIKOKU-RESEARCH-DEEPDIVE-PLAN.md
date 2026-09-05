TEIKOKU RESEARCH DEEPDIVE PLAN (layer-3 deep research for the research machine)
date: 2026-07-07
home: WSL teikoku distro, ~/teikoku/conductor (all code units land there; this file is the F:-side plan doc)
predecessor: workflow/reasoning/TEIKOKU-RESEARCH-MACHINE-PLAN.md (machine LIVE; this plan ADDS a depth layer, never rewrites shipped units)
PROOF_SPINE: none (teikoku personal research tooling; spine-exempt class pending owner ack per PSR-2)

PROBLEM
The live machine is triage, not research: each item gets one glm-5.2 call over a 2KB feed excerpt
(research_classify.py EXCERPT_CAP=2048, CLASSIFY_MAX_TOKENS=512) and steal survivors get one 700-token
brief whose URL enrichment is OFF by default (research_brief.py ENRICH_ENABLED=0; its docstring calls
multi-source consolidation "a v1 simplification"). Zero corroboration, zero source tiers, zero
contradiction surfacing. This plan ports the cheap-to-build parts of ELAI's research doctrine
(docs/thefinalglance/ELAI-SYSTEMS-OVERVIEW.md sections 8/10/14) and the oracle skill's floor discipline
into the machine, teikoku-style: python owns the fetching and the floors, the model only synthesizes
over a python-assembled source pack, and every miss is stamped honestly.

DESIGN (grounded steals, cheapest-first)
1. Deterministic 5-tier source-quality lookup (ELAI overview sec 8 "Source-quality is a deterministic
   5-tier lookup"): a python domain->tier table. T1 arxiv.org/official docs, T2 github.com/HF/vendor
   engineering blogs, T3 HN/reddit/lobsters, T4 personal blogs/medium/substack, T5 unknown. No LLM.
2. FlashResearch breadth decomposition (sec 8): deterministic sub-query templates per verdict kind
   ("<kw> reproduction", "<kw> criticism limitations", "<kw> vs alternatives", "site:github.com <kw>"),
   breadth cap 5 queries x top-2 results, fetch cap 8 pages x 8KB, skip a result whose domain already
   appears twice in the pack (info-gain pruning, simplified).
3. Typed claim extraction, structurally validated (sec 14 "parsed structurally, never by asking a model
   what it claimed"): synthesis returns claims with source_ids; python drops any claim citing a source
   id not in the pack and counts the drops (dropped_uncited, journaled).
4. Cross-family skeptic (sec 10 mode B + sec 14 P1 evidence round-trip): a second call on a DIFFERENT
   model family (kimi-k2-7, already in RESEARCH_MODEL_CHAIN at research_dispatch.py:61) tries to refute
   the top claims against the same pack. Disputed claims are flagged, never silently kept.
5. Oracle-style floors, honest degrade (oracle SKILL.md floors; wiring doctrine R1 missing-evidence=FAIL):
   floors = >=4 sources fetched OK, >=2 distinct domains, >=2 tiers represented, >=1 corroboration query
   fetched. Unmet -> the page is stamped DEPTH: FLOOR-NOT-MET(<reasons>). Budget/backpressure defer ->
   DEPTH: triage. Never silently thin.
6. Injection hardening carried over: pack pages are untrusted (same fence discipline as classify), plus
   a NEW frozen canary fixture: a corroboration page with a planted instruction must not move any enum
   or cause a fetch of a planted URL.
Scope guard: deep pass runs ONLY for steal-scan survivors past the 0.8 gate (a handful per day, queue
cap 30). Trend/watchlist items keep the cheap triage path. The weekly strong-model batch review stays
the judgment layer above this (unchanged, separate deferred item).

GROUNDED (verified before authoring, no recon needed)
- SearXNG /search?format=json returns parseable JSON for arbitrary queries: curled live 2026-07-07
  against http://127.0.0.1:8889 inside the distro (teikoku-searxng unit; research_scout.py already uses it).
- MODEL_CHAIN = glm-5-2,kimi-k2-7,swe-1-6 (research_dispatch.py:61); family-token self-id matching
  exists (research_dispatch.py:346).
- Fetch seams _http_get/_http_json/_excerpt exist and are injectable-by-convention (research_fetch.py:66,80;
  research_scout.py passes http_json= as a test seam).
- brief() fallback pattern + fence discipline exist (research_brief.py; research_classify.UNTRUSTED_PREAMBLE).
- Budget arbiter + CycleBudget exist (research_dispatch.py:311, TOKEN_BUDGET_PER_CYCLE=60000).
Cost envelope: one deepdive = ~8 free HTTP fetches + 2 LLM calls (~3K in / 700 out each) ~= 8K est
tokens; cap 2 per cycle = ~16K of the 60K cycle budget. Fits without starving classify.

UNITS (vertical slices; producer + production consumer + smoke land together per unit)

--------------------------------------------------------------------------------
U-R1 SOURCE PACK ASSEMBLER + TIER TABLE (research_sources.py, wired into the brief prompt)
--------------------------------------------------------------------------------
Deliverables: (1) research_sources.py: TIER_TABLE, query_templates(kind, keywords), build_source_pack(item,
verdict, http_get=, http_json=) -> {sources:[{id,url,domain,tier,query,text<=8KB}], floors:{met,reasons}};
(2) research_brief.build_brief_prompt consumes the pack (numbered <<<SRC id=N tier=T>>> fences) when one
is passed; (3) unit tests with canned fixtures (no network).
MOVE 1: flip RESEARCH_BRIEF_ENRICH=1 in the teikoku-research.service unit file (interim fallback depth;
  edit the unit itself, NOT a drop-in, per the 2026-07-06 landmine). Once MOVE 3 lands, the pack's
  primary-source fetch duplicates _enrich()'s item-URL fetch on the pack path: MOVE 3 therefore skips
  _enrich() whenever a pack is passed (one fetch per URL); the flag stays ON for the no-pack fallback only.
EXPECTED_OBSERVATION: systemctl show teikoku-research -p Environment lists RESEARCH_BRIEF_ENRICH=1 after
  daemon-reload; service restarts clean (systemctl is-active = active).
ON_FAIL: service fails to start -> cause is a malformed Environment= line -> revert the one line, restart,
  re-add with exact KEY=VAL syntax; the machine must never stay down over an env flag.
MOVE 2: author research_sources.py per the design (stdlib urllib only via research_fetch seams; SearXNG at
  SEARXNG_URL env default http://127.0.0.1:8889; primary source = the item url itself fetched first; HN
  items also pull the algolia comment tree per RECON R1; tier table + floors + info-gain domain skip).
EXPECTED_OBSERVATION: pytest tests/test_research_sources.py green: pack from fixtures has >=4 sources,
  tier tags match the table, floors.met flips false when fixtures starve a floor, planted-URL fixture is
  never fetched (fetch log assertion).
ON_FAIL: floors flap on fixture ordering -> cause is nondeterministic dict/result ordering -> sort sources
  by (tier, domain, url) before id assignment; ids must be stable for the citation gate.
MOVE 3: wire the pack into build_brief_prompt (new optional pack= param; sources fenced and numbered; the
  old excerpt-only path stays as the no-pack fallback) and call build_source_pack from the steal-survivor
  brief call site (exact site per RECON R2).
EXPECTED_OBSERVATION: a scratch-vault run-once --classify on a live steal topic produces a steal page whose
  body carries a SOURCES section with >=4 tier-tagged entries, and the journal row carries
  deepdive.sources_fetched >= 4 (or floors_not_met reasons).
ON_FAIL: live SearXNG returns zero results for template queries -> cause is over-specific keywords ->
  keyword extraction falls back from title bigrams to single salient tokens; if still zero, floors stamp
  FLOOR-NOT-MET(corroboration) and the run continues (honest degrade, never a hang).
FORK: if RECON R2 shows brief() is called from more than one production site, wire the pack into the
  steal-survivor site ONLY; trend/watchlist callers keep the no-pack path. Trigger: grep shows >1 caller.
FORK: if the HN comment-tree endpoint (RECON R1) is unreachable or shape-changed, the comments leg is
  dropped; floors do not count it (comments are bonus, never a floor input). Trigger: R1 curl fails.
RECON_NEEDED R1: settle the HN comment endpoint: curl -s https://hn.algolia.com/api/v1/items/1 | head -c 200
  (expect JSON with children[]). Run before MOVE 2.
RECON_NEEDED R2: settle the brief() production call site: grep -n "brief(" ~/teikoku/conductor/research_ingest.py
  ~/teikoku/conductor/research_dispatch.py. Run before MOVE 3.
PRODUCTION_HOOK: the steal-survivor brief call site in research_ingest (per R2) — build_source_pack has its
  real production caller there in THIS unit; a pack builder with only test callers is a BLOCKED return.
VERIFY: pytest tests/test_research_sources.py + existing research suites (no regression vs the current
  green count per RECON R4a); off-agent artifacts = the scratch steal page file (auditor greps SOURCES
  section) + the journal.jsonl row fields (python-authored, not model output).
ABORT: existing research tests regress and the cause is not a deliberately updated assertion; or
  research_sources.py needs a third rewrite for the same failing test (anti-spiral).

--------------------------------------------------------------------------------
U-R2 CLAIMS + CITATION GATE + CONTRADICTIONS (research_deepdive.py synthesis)
--------------------------------------------------------------------------------
Deliverables: (1) research_deepdive.py: build_deepdive_prompt(item, verdict, pack) + synthesize(...) ->
{brief, portable, take, claims:[{claim, source_ids, kind}], contradictions:[{point, a_source_id,
b_source_id}], confidence} with _validate_citations dropping uncited claims (dropped_uncited counter);
(2) research_ingest._steal_body renders EVIDENCE (claims with [S<n>] citations) + CONTRADICTIONS +
SOURCES + DEPTH sections; (3) unit tests incl. a citation-gaming fixture.
MOVE 1: author research_deepdive.py; same one_shot + fenced-JSON + fallback discipline as research_brief
  (parse failure degrades to the U-R1 pack-aware brief, stamped DEPTH: brief-fallback). synthesize() is
  called by research_ingest._run_brief IN PLACE OF research_brief.brief() when a pack is present —
  brief() remains only the no-pack fallback, never both on one survivor. synthesize() inherits _run_brief's
  EXISTING budget.spent() short-circuit + arbiter.acquire()/release() wrap (research_ingest.py:574-586,
  the D-4 chat-backpressure invariant) in THIS changeset — the guard is not deferred to U-R4.
EXPECTED_OBSERVATION: pytest tests/test_research_deepdive.py green: fixture synthesis yields >=2 claims
  each citing a real source id; a fixture response citing a nonexistent S9 has that claim dropped and
  dropped_uncited==1; a response that is not fenced JSON degrades to the brief fallback with ok=False;
  a call-count spy asserts exactly ONE synthesis LLM call per non-skeptic survivor turn (never brief()+
  synthesize() both) and zero calls when budget.spent() is true.
ON_FAIL: glm cannot hold the schema (frequent parse failures live) -> cause is schema width -> shrink to
  claims+contradictions only (brief/portable/take stay on the U-R1 brief path) and re-measure; if parse
  failures stay >50% over 10 live survivors, ABORT the unit and keep U-R1 depth only.
MOVE 2: render the new sections in _steal_body and stamp DEPTH: deep | brief-fallback | triage |
  FLOOR-NOT-MET(<reasons>) on every steal page.
EXPECTED_OBSERVATION: scratch-vault live run: steal page shows EVIDENCE with [S1]-style citations that
  resolve to SOURCES entries, and the journal row carries deepdive.claims, deepdive.dropped_uncited.
ON_FAIL: page renders but citations point at pruned sources -> cause is id assignment after pruning ->
  assign ids only after the pack is final (U-R1 MOVE 2 stable-sort guarantee); add a regression test.
FORKS: none
RECON_NEEDED R4a: record the current green research-suite test count before touching anything:
  cd ~/teikoku/conductor && python3 -m pytest tests/ -q --ignore=tests/test_chat_server.py
  --ignore=tests/test_temporal_backbone.py 2>&1 | tail -1 (those two files have PRE-EXISTING collection
  errors unrelated to this plan, verified 2026-07-07; without the ignores the bare command yields no
  count). The number is the no-regression baseline; every VERIFY in U-R1..U-R5 uses the SAME ignore set.
RECON_NEEDED R4b: read the journal row writer before adding deepdive fields (grep -n "journal" research_dispatch.py,
  read the row dict) so field names extend, never collide.
PRODUCTION_HOOK: research_ingest.file_steals_entry -> _steal_body (the existing steal-page producer);
  synthesize() is called from the same steal-survivor path U-R1 wired, in THIS changeset.
VERIFY: pytest tests/test_research_deepdive.py + full research suites >= R4a baseline; off-agent artifacts =
  the scratch steal page (EVIDENCE/CONTRADICTIONS/DEPTH sections grep) + journal row fields.
ABORT: parse-failure rate >50% over 10 live survivors after the MOVE 1 counter-move (keep U-R1, drop U-R2,
  record in handoff); or any brain_store consumer regression (test_brain.py must stay green untouched).

INTEGRATION_SMOKE SMOKE-A (after U-R2, R10 mid-cadence): an INDEPENDENT agent (not the executor of U-R1/2)
drives the REAL entrypoint on a HARD input: research_dispatch.py run-once --classify --topic elai-steal-scan
against a scratch vault with live bridges, plus the frozen injection-canary fixture pack. PASS = a steal
page lands with SOURCES(>=4, >=2 tiers) + EVIDENCE(citations resolve) + DEPTH stamp; journal row carries
deepdive fields; the planted instruction moved no enum and triggered no planted-URL fetch; real /mnt/c
vault byte-untouched. Missing any artifact = FAIL, never SKIP.

--------------------------------------------------------------------------------
U-R3 CROSS-FAMILY SKEPTIC PASS (research_deepdive.skeptic_review)
--------------------------------------------------------------------------------
Deliverables: (1) skeptic_review(pack, claims, lane_pool) -> per-claim {verdict: supported|disputed|unclear,
counter, source_id} on kimi-k2-7 (different family than the glm synthesizer); (2) CORROBORATION section on
the steal page + disputed flag on the steals-queue line + digest; (3) unit tests + a same-family-degrade test.
MOVE 1: author skeptic_review; pin kimi-k2-7 via the lane/model-chain mechanism (exact pinning API per
  RECON R3; DevinLane/LanePool already accept model= at construction per research_dispatch.py:373-376,
  512-524 — the FORK below is the fallback if per-call pinning is absent); prompt = the same fenced pack
  + the claims list + "refute if you can, cite the source id". skeptic_review runs under the SAME
  budget.spent() + arbiter.acquire()/release() guard as synthesize (this changeset, not U-R4).
EXPECTED_OBSERVATION: pytest green: fixture skeptic response flips one claim to disputed with a counter
  quote citing a valid source id; invalid-id counters are dropped (same citation gate).
ON_FAIL: kimi pin fails self-id or the lane 504s -> cause is bridge model routing -> ONE retry, then degrade
  to a same-model skeptic labeled CORROBORATION: same-family (weak) on the page (mirrors ELAI's adversary
  same-family refusal by honest LABELING, since teikoku has one pool). Never silently skip the pass.
MOVE 2: render CORROBORATION on the page; disputed claims prefix their EVIDENCE line with [DISPUTED];
  journal row gains deepdive.disputed count; the steals-queue entry line carries a disputed marker so the
  weekly batch review sees it without opening the page.
EXPECTED_OBSERVATION: scratch live run where the skeptic disputes >=1 claim shows [DISPUTED] on the page
  AND the queue line marker AND deepdive.disputed>=1 in the journal row, all three consistent.
ON_FAIL: verdicts render but the counts disagree across page/queue/journal -> cause is three renderers
  computing independently -> compute once in synthesize-return, thread the same dict to all three.
FORK: if RECON R3 shows one_shot cannot pin a model per-call, route the skeptic through a dedicated lane
  env (RESEARCH_SKEPTIC_MODEL=kimi-k2-7 consumed at lane acquisition). Trigger: no per-call model param.
RECON_NEEDED R3: settle per-call model pinning: grep -n "def one_shot\|model" the lane class used by
  research_dispatch (read the signature). Run before MOVE 1.
PRODUCTION_HOOK: the same steal-survivor path (synthesize -> skeptic_review -> _steal_body) in this
  changeset; skeptic_review with only test callers is a BLOCKED return.
VERIFY: pytest tests/test_research_deepdive.py::TestSkeptic + suites >= R4a baseline; off-agent artifacts =
  page CORROBORATION grep + journal deepdive.disputed + the queue line marker.
ABORT: skeptic adds >2 LLM calls per survivor in practice (budget breach; it is specced at 1) or the
  disputed rate is 100% over 10 live survivors (the skeptic prompt is broken, not the claims).

--------------------------------------------------------------------------------
U-R4 DISPATCH WIRING: BUDGET, CADENCE, KILL-SWITCH, JOURNAL (research_dispatch.py + research_routes.py)
--------------------------------------------------------------------------------
Deliverables: (1) deepdive gating in the dispatcher: default ON, RESEARCH_DEEPDIVE=0 off-switch,
RESEARCH_DEEPDIVE_MAX_PER_CYCLE=2 (named constants, env-overridable), backpressure/degraded -> skip
deepdive with DEPTH: triage (the arbiter and organizer pause outrank depth); (2) journal row deepdive
fields finalized per R4b; (3) GET /research/findings and /research/for-you responses carry depth fields
(sources count, tiers, disputed, depth stamp) read from the vault pages.
MOVE 1: dispatcher gating + counters.
EXPECTED_OBSERVATION: unit tests green: budget.spent() short-circuits deepdive (depth=triage stamped);
  cap 2 enforced when 3 survivors arrive in one cycle (third gets triage); RESEARCH_DEEPDIVE=0 run never
  imports-calls build_source_pack (fetch log empty).
ON_FAIL: the cap starves the queue (survivors 3+ never deepen) -> cause is per-cycle cap with no carryover ->
  deterministic backfill with a NAMED state source: re-scan the vault steals pages for frontmatter
  depth: triage AND status: queued, sorted by created_at ascending, serve the oldest first — and re-check
  status: queued immediately before reprocessing (the weekly batch review can promote/reject/archive an
  entry between cycles; a non-queued entry is NEVER re-touched, D-15). Tested with a fixture where an
  entry flips to archived between cycles.
RECON_NEEDED R6: settle where the batch-review dequeue/archive status lives and is readable (grep the
  steals-queue page frontmatter + research_ingest.file_steals_entry field set) BEFORE implementing the
  backfill re-check. Run before MOVE 1.
MOVE 2: route depth fields through research_routes (parse the DEPTH/SOURCES/CORROBORATION sections the
  same way _parse_leaderboard_body parses tables).
EXPECTED_OBSERVATION: curl http://127.0.0.1:8787/research/findings shows depth fields on a deepdived item
  and their absence (not nulls-as-lies) on triage items.
ON_FAIL: parser misreads legacy pre-plan steal pages -> cause is pages without the new sections -> absent
  sections parse to depth="triage-legacy", never an exception (test with a real pre-plan page fixture).
FORKS: none
RECON_NEEDED: R6 above; R4b settled in U-R2.
PRODUCTION_HOOK: cmd_loop / run_topic_cycle (the live service path) for the gating; slice_api do_GET for
  the route fields — both wired in THIS changeset.
VERIFY: pytest suites >= R4a baseline; off-agent artifacts = journal rows with deepdive fields from a live
  scratch cycle + the /research/findings JSON (curl output the auditor runs itself).
ABORT: any change to the arbiter/organizer-pause semantics (this unit READS them, never edits them); if
  gating cannot be added without touching arbiter internals, STOP and hand off.

--------------------------------------------------------------------------------
U-R5 UI SURFACING (signal/src: research.ts + ResearchTab.svelte + ForYouCards.svelte)
--------------------------------------------------------------------------------
Deliverables: (1) research.ts types + fetchers extended with the depth fields; (2) findings cards show a
source-count + tier chip row and a DISPUTED badge; DEPTH: FLOOR-NOT-MET renders as a dim "shallow" tag
(honesty visible, not hidden); (3) svelte-check + vite build green.
MOVE 1: extend types + card rendering (read the current card markup per RECON R5 before editing).
EXPECTED_OBSERVATION: svelte-check 0 errors, vite build green, and the running app (after a native
  relaunch) shows tier chips on a deepdived finding and nothing (no empty chip row) on triage findings.
ON_FAIL: chips render on stale cached findings with no depth fields -> cause is undefined-vs-absent
  handling in TS -> depth fields optional in the type, chip row gated on presence, regression-checked
  against a legacy findings payload fixture.
FORKS: none
RECON_NEEDED R5: read the findings card block in ResearchTab.svelte (offset/limit the card region) before
  the edit; the card structure changed 2026-07-07 (leaderboard tables landed).
PRODUCTION_HOOK: ResearchTab.svelte findings view (the shipped default tab) — the new fields render on the
  live surface in THIS changeset, not behind an unreachable component.
VERIFY: svelte-check 0/0 + vite build green (off-agent: the auditor runs both); owner-eyes on the running
  app is the final pixel check (explicitly listed UNVERIFIED until the owner looks).
ABORT: any edit needed outside the three named files + capabilities (scope creep -> stop and hand off).

--------------------------------------------------------------------------------
INTEGRATION_SMOKE SMOKE-B (final, R10): an INDEPENDENT agent (never a U-R executor) drives the REAL
service path end-to-end on a HARD input: enable the service against a scratch vault, let cmd_loop run a
full cycle on elai-steal-scan + one recommend topic with live bridges, with (a) the frozen injection
canary planted in a corroboration fixture, (b) a forced-kimi-unavailable window to prove the same-family
degrade label, (c) budget forced low to prove the DEPTH: triage defer, (d) a survivors>cap cycle to prove
deterministic backfill. PASS = all four observable on pages/journal/routes; the real /mnt/c vault
byte-identical before/after; existing 36/36 machine tests + full suite >= R4a baseline; frozen canary
fixtures unmodified (gate-freeze posture). Any missing artifact = FAIL. Never run two model instances
beyond the existing lane pool (no sidecar boot involved).

--------------------------------------------------------------------------------
PLAN-LEVEL
--------------------------------------------------------------------------------
ABORT (plan-level): the same module rewritten >3 times for the same failing gate (anti-spiral -> persona
switch or hand back); any test in test_brain.py regressing (brain_store is a consumer contract, frozen to
this plan); any edit to the arbiter, organizer window, classify gate thresholds, or the canary fixtures
(out of scope, gate-frozen posture); est deepdive cost measured >8K tokens per survivor live (re-scope the
pack caps before continuing).
VERIFY (plan-level, the auditor's off-agent close-out): full conductor suite >= R4a baseline; SMOKE-A and
SMOKE-B artifacts on disk; journal rows for every live assertion; py_compile on every touched .py;
svelte-check + vite build for U-R5. The executor's self-runs are never the authority.
FORK (plan-level): if the owner declines the default-ON posture for deepdive, flip to default-OFF and add
the promotion row teikoku-side (the wire-dark-allowlist analog is the WIRE-DARK.md convention already used
in the conductor). Trigger: owner says so at review.

REDTEAM
Attack 1 (author self-attack, pre-audit): the synthesizer games the citation gate by citing S1 for every
claim — the gate only checks existence, not relevance. Result: PARTIAL SUCCESS -> patched: SMOKE-A asserts
>=2 distinct source_ids across the claim set of a multi-source pack, and the U-R3 skeptic reads the SAME
pack (a wrong citation is exactly what a refuter catches); dropped_uncited + disputed are both journaled
so a gamed brief is visible in the row, not just the page.
Attack 2 (author self-attack, pre-audit): a hostile page in the corroboration set instructs the model to
mark all claims supported and cite it. Result: FAILED against the plan as written — the skeptic runs on a
different model family with its own fence, the enum gate already sanitizes verdict fields, and the frozen
canary fixture in SMOKE-A/B asserts exactly this class; a new canary variant covering the "cite me"
instruction is added to the fixture set in U-R2.
--- adversarial-auditor mode=plan, 2026-07-07 (independent, non-author; verdict BLOCK -> patched, see below) ---
Attack 3 (auditor): U-R2/U-R3 new LLM calls ship before U-R4's gating, running unmetered on the shared
3-lane pool and defeating the D-4 chat-backpressure invariant for that window. SUCCEEDED -> patched:
synthesize() and skeptic_review() inherit _run_brief's existing budget.spent() + arbiter.acquire()/release()
guard in their OWN changesets; U-R4 only adds the per-cycle cap + DEPTH stamping on top.
Attack 4 (auditor): replace-vs-additive ambiguity — synthesize() superset of brief() with no MOVE stating
substitution; both firing doubles cost and duplicates page fields. SUCCEEDED -> patched: U-R2 MOVE 1 now
states synthesize() replaces brief() when a pack is present (brief() = no-pack fallback only) + a
call-count spy asserts exactly one synthesis call per survivor.
Attack 5 (auditor): R4a baseline command broken on the live tree (tests/test_chat_server.py +
tests/test_temporal_backbone.py have pre-existing collection errors; bare pytest yields no count).
SUCCEEDED -> patched: R4a carries the exact --ignore set; all downstream VERIFY counts use the same set.
Attack 6 (auditor): backfill mechanism unnamed — could re-touch an entry the weekly batch review already
promoted/archived (D-15 violation, double-file risk). SUCCEEDED -> patched: backfill state source named
(vault scan for depth: triage + status: queued, created_at ascending) + a status re-check immediately
before reprocessing + RECON R6 settles where that status lives.
Attack 7 (auditor): SearXNG JSON shape assumption. FAILED — research_scout.discover_via_searxng already
parses the identical endpoint live; the plan reuses a proven seam.
Attack 8 (auditor): per-call model pinning for the skeptic might not exist. FAILED — DevinLane/LanePool
accept model= at construction (research_dispatch.py:373-376, 512-524), exactly the fallback U-R3's FORK
anticipates.
Auditor patch 5 (enrich/pack duplicate item-URL fetch) also applied at U-R1 MOVE 1.
(this block is append-only; further audits append below.)
