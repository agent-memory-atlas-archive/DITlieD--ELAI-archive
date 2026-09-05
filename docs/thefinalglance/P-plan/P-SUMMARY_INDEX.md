# P-plan — Summary Index of all 25 sessions (Tracks PERSONAL-INCARNATION + COMMAND-CENTER + COMPUTER-USE-STEAL + OPENCLAW-STEAL + CREATIVITY-FALSIFIER)
Generated: 2026-07-07 (P-1..P-8); command-center P-9..P-11 2026-07-08/09; steal tracks P-12..P-22 2026-07-11. Mirrors O-SUMMARY_INDEX / N-SUMMARY_INDEX format.

Each entry preserves: the lever + source finding (digest GAP-1..GAP-8), grounding strength, the shipped subsystem
it EXTENDS (with the plan-id), the genuinely-new mechanism it ADDS, the COMPOSES-WITH artifacts consumed, the
production entrypoint, and the observable_effect done-gate. Doctrine + sequencing live in P-0_INDEX.md; the source
design is workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md.

## Premise
The owner's "ELAI is the way a person builds their own WORLD — nodes, connected systems, everything
protected/organized/committed, integrating with existing brains, one seat scaling from a solo toy to many
connected small projects to a big multi-person project" decomposes into eight vertical slices that LIFT the
existing trust spine ONE GRAIN (changeset->app, task->fleet, seat->world, org->household). Each is ~80%
composition of shipped subsystems; the differentiator vs every actor in the bespoke-tools thread is the same in
every unit: receipts + taint + replay + cant-lie + wiring gates, at the grain the thread actually lives at. P is a
product-surface letter, PROOF_SPINE: none (owner-acked); it feeds no PS-1..PS-6 number and is authored-now /
executed-after-the-spine (A-7).

==== TRACK PERSONAL-INCARNATION ====

### P-1 — app-fleet layer (health-tick + gated auto-repair) [KEYSTONE]
Track PERSONAL-INCARNATION | lever fleet-grain-over-taskspec | grounding MODERATE (index-derived, PLAN-asserted) |
  nature ~80% composition | effort M-L | WAVE 1
Steal/build: no corpus letter owns "the user's N bespoke apps" as first-class objects — the thread's biggest pain
  (T2 keep-it-alive, florinandrei "100% on you now"). ADDS an AppManifest (purpose + data namespaces + ports +
  smoke + RRA lineage + dep manifest) + a FleetRegistry; a health tick is an O-2 TaskSpec (NOT new machinery);
  per-app drift canaries GENERALIZE G-10; dep-rot points F-9 at each manifest; a red tick proposes a fix that
  passes the SAME wiring/claim gates (gated auto-repair, never a silent patch), rendered human-first via N-7.
COMPOSES-WITH: O-2 TaskSpec + DashboardInbox (EXTEND), G-10 canary (generalize), F-9/F-5, L-14 RRA, N-7, M-4 pattern.
Integration: crates/orchestrator AppManifest table + FleetRegistry + health-tick TaskSpec factory; `elai fleet` CLI.
observable_effect: register an app with a broken smoke, advance the task_runner tick; one fleet_health_tick row
  {app_id, verdict=red, red_reasons} lands off-agent exactly once + a red-tick N-7 inbox card; NEGATIVE: a green
  tick that still cards, or a red tick with empty red_reasons, FAILS.

### P-2 — inter-tool ports + world-grain event bus
Track PERSONAL-INCARNATION | lever typed-ports-over-canvas | grounding MODERATE (index-derived, PLAN-asserted) |
  nature mostly composition; the port-schema registry is the genuine build | effort M | WAVE 2 | depends P-1,O-4
Steal/build: T3 glue (Plane_Garbage "requires a bit of glue"; wtjones "modular tools and glue them together").
  ADDS a PortSchema on the AppManifest + a PortRegistry; the O-4 canvas wires tool->tool EDGES carrying an N-6
  provenance tier + G-1 taint; a zero-consumer output renders VISIBLY DISCONNECTED (wire-dark-made-visual at world
  grain); M-5 elai-verify conformance makes a third-party tool a mountable node. NO live broker/runtime — dispatch
  stays the FSM (O-4 doctrine); the "bus" is a typed-edge projection + journaled invocation rows.
COMPOSES-WITH: P-1 AppManifest (EXTEND), O-4 canvas (EXTEND edges), N-6 lattice, G-1/G-2, M-5/M-6.
Integration: crates/orchestrator PortSchema + PortRegistry + compat checker; `elai fleet wire|ports` CLI; O-4 view (Tier-2).
observable_effect: wire a weather output into a wardrobe input; one tool_port_edge row {src.out, dst.in, n6_tier,
  taint_tier} on a bind; an incompatible wire lands NO row + refuses; NEGATIVE: an edge missing the taint tier FAILS.

### P-3 — persona/mechanism split + hand-this-box-to-a-friend export
Track PERSONAL-INCARNATION | lever mechanism-vs-persona-factoring | grounding MODERATE (index-derived, PLAN-asserted) |
  nature bundle=build, factoring=doctrine | effort M | WAVE 3 | depends P-1,P-2
Steal/build: T5 share; untransferability (heynoswearing "will immediately break if anyone else touches it") is a
  FACTORING failure, not a law. ADDS an AppBundle (mechanism + manifest + RRA receipt + G-9-redacted provenance);
  the PERSONA layer (rules rows, memory namespaces, thresholds, tokens — all DATA) is EXCLUDED and REGENERATED on
  the recipient's seat; `elai app import` re-verifies via M-5 and REFUSES a tampered bundle; admission-gated like
  skills (L-12/skillspector) — friend-to-friend and marketplace share one gate.
COMPOSES-WITH: P-1/P-2 (mechanism), M-5/M-6, L-12 admission, G-8/G-9, FS-1/L-13, E-5 + P-7 (persona regen).
Integration: crates/orchestrator AppBundle + export/import + M-5 verify + G-9 redaction; `elai app export|import` CLI.
observable_effect: export an app, tamper the bundle, import it; an app_export row on export; an app_import REFUSAL
  (no mount) for the tampered bundle + a mount row for the clean one; NEGATIVE: a tampered bundle that mounts, or
  an export leaking a secret, FAILS.

### P-4 — living external brains
Track PERSONAL-INCARNATION | lever mount-LIVE-not-static | grounding WEAK-until-RECON (Obsidian/Notion surfaces are
  training-only per digest; source-grounding is a hard precondition) | nature composition + per-store contract |
  effort M-L | WAVE 2 | depends O-3,P-6 | GATED
Steal/build: "integrate with existing brains". O-3 mounts a static markdown vault; real brains are LIVE (Obsidian
  under concurrent plugin edits, Notion, SQLite). ADDS a LivingStoreContract per store class (watch + change feed +
  F-5/N-6-tagged ingest, WRITE only inside O-3-claimed namespaces); B-35's context pipeline lifted to living
  stores; the D-5/O-3 life denylist becomes a hard routing wall (P-6). BLOCKING RECON on the Obsidian/Notion API
  before any code (the digest flags these UNGROUNDED); the notion MCP is a live grounding source this session.
COMPOSES-WITH: O-3 vault contract (EXTEND to live), B-35, F-5/N-6, D-5 denylist, P-6 lattice + E-20.
Integration: crates/knowledge LivingStoreContract + Obsidian watch adapter + O-3-claim-gated writer; `elai brain mount|sync` CLI.
observable_effect: mount a fixture Obsidian vault, edit a claimed-namespace note, attempt an unclaimed write; a
  vault_ingest row {note_path, n6_tier, staleness_ttl} on the edit + a claim_violation row on the unclaimed write;
  NEGATIVE: a mutation of a user-owned note, or an ingest missing tier/TTL, FAILS.

### P-5 — non-coder operator surface + graduated ceremony
Track PERSONAL-INCARNATION | lever graduated-ceremony-profile | grounding MODERATE (index-derived, PLAN-asserted) |
  nature ~90% composition | effort M | WAVE 1
Steal/build: T1 for the non-coder (Hapiro "absolutely 0 understanding of coding"). The trust spine must scale
  DOWN: a spell-card generator is not a trading strategy. ADDS a CeremonyProfile (toy|personal|project|critical)
  on K-5/K-4: at the toy tier gates run OBSERVE-ONLY (still run + journal, never block), receipts collapse to one
  line that INDEXES the full journal, nothing silently skipped (W-2/R1); the operator surface is N-7 pointed at
  the P-1 fleet grain (a red tick = one sentence + one gated repair button). The toy-tier gate tax is MEASURED
  near-zero (A-3 falsifier).
COMPOSES-WITH: K-5/K-4 (EXTEND with a tier profile), N-7 (new fleet-grain target), P-1 red tick, GD-4/W-2.
Integration: crates/orchestrator CeremonyProfile + K-5 gate-mode wiring + one-line receipt; `elai profile set|tax` CLI.
observable_effect: set toy, run a gated fleet action, then the same at critical; a ceremony_profile_set row + the
  verdict journaled at BOTH tiers; toy does NOT block on WARN, critical DOES; the one-liner resolves to the full
  journal; `elai profile tax --tier toy` near-zero; NEGATIVE: a toy tier that drops the journal row FAILS.

### P-6 — life-data privacy lattice
Track PERSONAL-INCARNATION | lever out-flow-privacy-dimension | grounding MODERATE (index-derived, PLAN-asserted) |
  nature composition (extend N-6, no second lattice) | effort M | WAVE 2 | depends N-6,O-3
Steal/build: T6 owned-and-private (Azko87 "an app I don't need to login to"; the birdwatcher app on two phones).
  ADDS a domain (life|work|shared) DIMENSION on N-6's ONE lattice + per-tool read_grants (default-deny), enforced
  at the O-3 vault-read boundary AND E-20 routing; taint's MIRROR (owned bytes flowing OUT, vs G-1's untrusted
  bytes flowing IN); every grant N-7-rendered + journaled + revertible (H-49). A-1/N-2: a second privacy lattice
  is an authoring FAIL.
COMPOSES-WITH: N-6 (EXTEND), N-2 (authoring gate), G-1 (mirror), G-9, E-20, O-3 boundary, N-7, H-49.
Integration: crates/knowledge domain dimension + read_grants + O-3-read enforcement; `elai privacy grant|revoke|show` CLI.
observable_effect: grant budget-tool on budget-ns, then read budget-ns AND health-ns; a privacy_grant row; the
  budget-ns read succeeds, the health-ns read lands a privacy_read_refusal row; N-2 confirms one lattice;
  NEGATIVE: an ungranted read that succeeds, or a second lattice table, FAILS.

### P-7 — per-user operating profile
Track PERSONAL-INCARNATION | lever measured-asymmetric-process-adaptation | grounding MODERATE (index-derived,
  PLAN-asserted) | nature composition on N-3/H-17/H-18 | effort M-L | WAVE 3 | depends N-3 landed
Steal/build: the direct answer to "how does ELAI adapt to every user, not only memory/code/workflows". Memory
  adapts CONTENT (E-5); nothing adapts PROCESS. ADDS an OperatingProfile (autonomy/strictness/verbosity per
  domain) learned + promoted through N-3/H-17/H-18 (measured, never a vibe); the load-bearing invariant is
  ASYMMETRY (M-2 at process grain): tighten free, loosen ONLY on measured OWN-RUN FP evidence — the structural
  defense against adaptation-as-sycophancy (A-4). Observe-first (GD-4), journaled, revertible (H-49). The three
  archetypes (solo toy / connected / team) = three DEFAULTS on ONE engine, not three products.
COMPOSES-WITH: N-3/H-17/H-18 (EXTEND to process metrics), M-2 (mirror asymmetry), GD-4/H-49, E-5 (disjoint),
  P-5/P-2/M/P-8 (the defaults' targets).
Integration: crates/orchestrator OperatingProfile + asymmetry-guarded change fn (loosen via H-17/H-18); `elai profile show|tighten|loosen` CLI.
observable_effect: loosen with no evidence (REFUSED), then with measured own-run FP evidence (PROMOTED via H-18),
  and tighten (applies); the refusal + promotion + a revert all journaled with evidence_ref; NEGATIVE: a loosen on
  no/raw-count evidence, or on another user's data, FAILS.

### P-8 — small-team collaboration grain
Track PERSONAL-INCARNATION | lever shared-world-2-to-10-seats | grounding MODERATE (index-derived, PLAN-asserted);
  the merge kernel is an OPEN design problem | nature composition-heavy + one hard design kernel | effort L |
  WAVE 4 | depends O-3,K-23
Steal/build: the "big multi-person project" leg. M solves org COMMERCE (no org SaaS); K-23 solves multi-repo
  memory for ONE seat. The missing grain is 2-10 HUMANS + their agents in ONE world. ADDS shared vault namespaces
  (O-3 already anticipates two claimants), cross-seat locks lifted from harness tooling to the product, per-seat
  {seat_id, agent_id} provenance on every SSB row (attribution is a query), M-3 rollup at a shared project. The
  HARD KERNEL, done as a SPIKE FIRST: a lossless, deterministic, causally-ordered two-seat SSB-stream MERGE (two
  seats' streams have NO defined join today) — NO last-writer-wins, NO org control plane, local-first.
COMPOSES-WITH: O-3 namespace-claims (EXTEND to N seats), K-23, I-2, M-2/M-3 rollup, harness cross-session locks, Phase 3H merge.
Integration: crates/orchestrator+knowledge SSB-stream merge + seat_id/agent_id columns + cross-seat lock; `elai world join|merge|whodid` CLI.
observable_effect: two seats write interleaved + concurrent events, merge BOTH directions; every row carries
  {seat_id, agent_id}; a seat_journal_merge row {seats, event_count, order_hash} with order_hash IDENTICAL
  regardless of direction; NEGATIVE: a dropped event, a direction-dependent order_hash, or a lost concurrent
  write, FAILS.

## Track cross-cutting (what makes it different)
Every unit enforces the SAME digest patches as stub_ban + REDTEAM: A-1 (no second lattice/scheduler — pass N-2 at
authoring, ride N-4/N-6/N-11); A-3 (ceremony tax must be MEASURED near-zero at the toy tier or GAP-5 fails); A-4
(adaptation loosens ONLY on measured own-run FP evidence — never user affect); A-5 (two-masters: substrate ships
as code, persona/packs/profiles are data, no per-user/per-recipient/per-seat code fork). A-2 (default-off
graveyard) is the whole-feature falsifier: every P-N ships its measured number from day one (fleet-health green %,
wired-edge count, exported-tool re-verify success rate, toy-tier gate tax, profile-promotion FP rate, merge
determinism) — no number, no claim (KD-1). A-6 (the "next model" objection) is the surviving-attack entry: habits
live in journaled DATA, G-10 watches the model, B-37 swaps it — the attack survives only as the P-1 RECON
(G-10 per-app canary generalization is asserted, not yet designed). A-7 (scope honesty): P is authored now,
executed after the proof spine.

==== TRACK COMPUTER-USE-STEAL ====
Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (spec-eval of ELAI's canvas + e2e-testing surface
vs 6 computer-use repos; steal list P1..P8 de-duped against the openclaw pass into five slices). Premise: the six
comparators are complements/operator-lanes not rivals (the I-40 `elai wrap` inversion — a comparator that wins on
raw capability becomes a taint-tagged journaled backend under ELAI's gates), but they expose ONE genuine
design-scope gap: no desktop/GUI operator lane, no agentic-run->replayable-artifact instance. Every ELAI anchor is
index-derived/PLAN-asserted (executor re-greps, W-7).

### P-12 — desktop/GUI operator lane [NEW; the honest design gap]
Track COMPUTER-USE-STEAL | lever gui-actuation-under-gates | grounding MODERATE (comparator source-verified in the
  eval doc; ELAI anchors PLAN-asserted) | nature NEW (no desktop computer-use lane in B..Q) | effort L | WAVE CU-2 | PROOF_SPINE none (product), owner-acked 2026-07-11
Steal/build: ELAI specs browser E2E (E-14) + media validation (4I) but has NO general desktop/GUI computer-use
  stage — a workflow node "drive this GUI app / test this game build" has no home. ADDS an OperatorLane: a
  disposable sandbox (cua's Windows-Sandbox/lume/Docker substrate VENDORED, MIT — cua's own in-loop safety is a
  commented-out TODO, comparison ENTRY 10, so ELAI supplies the fail-closed gates) + a perception feed (P-13 +
  OmniParser Tier-3 pixel fallback fired only when the AXTree is empty/incomplete) + openclaw's frameId/display
  fail-closed binding (a coordinate action is authorized only while the exact sha256-hashed image the model saw is
  still in context; epoch-invalidated on compaction; never retargets to display 0; V4) + a two-phase arm-commit
  (journal the grant BEFORE exposing the capability) + dual expiry (lease TTL checked on sweep AND inline) + a
  per-action bounded screen-recording whose hash is an off-agent evidence row. Every operator action is a G-1 taint
  source and an operator lane, NEVER load-bearing infrastructure a workflow's correctness rests on.
COMPOSES-WITH: E-14 (browser E2E sibling/doctrine), J-1/J-8 (perception gateway — P-13 extends), G-1 taint (a new
  actuation source), G-2 RRA (the action-stream), 4F feasibility sandbox (the isolation substrate), H-13
  sanitizeAction (diff cua's OperatorNormalizer pure-fn action repair against ours). Do NOT re-implement the sandbox
  (vendor cua), the perception (P-13), or the taint engine (G-1).
Integration: new crates/orchestrator/src/operator/ (lane + sandbox adapter + frame-binder + arm/lease) + an O-2
  TaskSpec stage kind operator_action; `elai operator run` CLI + the O-4 canvas node. frontend_pair: operator-action
  rows + the screen-recording thumbnail in an existing drawer (decide at edit, W-4). License anchor: OmniParser
  icon_detect weights AGPL (Florence captions MIT) — re-verify at edit.
observable_effect: run an operator TaskSpec that clicks a sandboxed app; one operator_action row {frame_hash,
  action, lease_id} + one operator_frame_bind row land off-agent + a bounded screen-recording artifact whose hash
  matches the row; a coordinate action computed against a STALE/compacted frame is REJECTED (fail-closed) with no
  actuation; NEGATIVE: an action that actuates on a stale frame_hash, or an arm that exposes the capability before
  its grant row lands, FAILS.

### P-13 — perception provenance + set-of-marks id-indirection
Track COMPUTER-USE-STEAL | lever id-indirection-over-coords | grounding STRONG (OmniParser arXiv:2408.00203 measured
  70.5->93.8% grounding lift, verified in the eval doc; J-1 anchors PLAN-asserted) | nature EXTENSION | effort M | WAVE CU-1 | PROOF_SPINE none
Steal/build: J-1's gateway conditions bytes + gives AXTree/DOM text, but the model still has to point. ADDS
  set-of-marks ID-INDIRECTION on the AXTree node list (the VLM returns an integer id, the harness owns geometry —
  OmniParser's mechanism behind the 93.8%, works with zero pixel model) + a per-element SOURCE tag
  (axtree_exact | ocr_text | vlm_caption | pixel_yolo) that IS the trust boundary (slots into G-1 taint /
  grounding) + an interactability flag filtering the VLM candidate set + advisory-only captions for unnamed
  canvas/game icons + asymmetric OCR-into-element label fusion. Tier-3 pixel/YOLO fires only when the AXTree is
  empty/provably incomplete (games, WebGL, native dead-UIA). Counts NEVER come from a VLM (J-1 law unchanged); a
  per-crop caption never stands in for element identity (OmniParser S8 anti-pattern).
COMPOSES-WITH: J-1 PerceptionScaffold + J-8 upgrade schema (EXTEND with source/id-index fields, prefer-reuse — net
  0..+1 rows), J-77 perception-faithfulness (a mis-sourced element = a ghost citation), E-14 AXTree doctrine
  (confirmed, not challenged), G-1 taint (source-tag = provenance). Re-implements nothing in J-1/J-8.
Integration: crates/orchestrator/src/perception/ (J-1's, extend scaffold.rs with source + box-id fields + a Tier-3
  pixel adapter behind the empty-AXTree guard). frontend_pair: rides J-1's perception tile (source-tagged element
  breakdown). No new CLI (the gateway is a service seam consumed by E-14 + P-12).
observable_effect: run the perception scaffold on a canvas-rendered UI fixture with an empty AXTree; the element
  list carries integer ids + a source tag per element + the Tier-3 pixel leg fires exactly once (trace shows zero
  VLM count calls); a follow-up "click element 7" resolves by id not coords; NEGATIVE: a count answered from the VLM
  leg, or an element missing its source tag, FAILS.

### P-14 — explore-then-crystallize record->replay lane [FEEDS PS-5]
Track COMPUTER-USE-STEAL | lever agentic-run-mints-artifact | grounding STRONG (cua replay_env record->re-score
  verified HIGH in the eval doc; openclaw jsonl-replay is INPUT-through-mock NOT record->re-score, C4) | nature
  EXTENSION | effort M-L | WAVE CU-2 | depends P-13 | PROOF_SPINE feeds PS-5
Steal/build: the strongest single design idea in the eval doc. ELAI has the GENERAL form (C template crystallizer,
  J verified case graph, O-2 TaskSpecs) but no browser/GUI INSTANCE of "an agentic exploration run mints a
  deterministic replayable artifact". ADDS: an E-14/P-12 exploration run EMITS a deterministic Playwright (browser)
  / pyautogui (desktop) script AS an O-2 TaskSpec stage, with a G-2 RRA receipt BINDING the script to the
  exploration run that minted it; the replay of that script is the cheap zero-reasoning execution mode
  (~100% reliable, no LLM in the loop). A crystallized flow IS an O-2 TaskSpec (not new dispatch machinery). This is
  MORE deterministic than E-14's agentic-per-run baseline and directly compounds per-customer reuse (PS-5).
COMPOSES-WITH: E-14 (the exploration run source), O-2 TaskSpec substrate (the artifact IS a stage — EXTEND the stage
  kinds), G-2 RRA (the script->run binding), C compiled-pipeline/template crystallizer (the general form — compose,
  don't fork), P-13 (the perception feed the exploration uses). Prior art cited, not copied: cua replay_env,
  agent-zero skills/ scripts convention.
Integration: crates/orchestrator O-2 stage kind crystallized_script + a script-emitter off the E-14/P-12 trace + the
  RRA binder; `elai e2e crystallize` / replay via the normal task_runner tick. frontend_pair: crystallized-stage
  node on the O-4 canvas (Tier-2). 
observable_effect: run an agentic E-14 exploration, then replay the minted stage; one crystallized_stage_minted row
  {script_hash, minting_run_id, rra_hash} + the replay drives the real entrypoint with ZERO model calls in the trace
  and reproduces the exploration's observable outcome; NEGATIVE: a crystallized stage whose RRA does not bind back
  to a real minting run, or a replay that silently invokes the model, FAILS.

### P-15 — verified case-graph hardening [FEEDS PS-5]
Track COMPUTER-USE-STEAL | lever gate-the-save-not-every-run | grounding STRONG (agent-s KnowledgeBase
  arXiv:2410.08164 verified; the ungated-save gap is agent-s's own shape; J-96/K-18 PLAN-asserted) | nature
  EXTENSION (of J-96, J CLOSED -> lands in P cross-linked) | effort M | WAVE CU-3 | PROOF_SPINE feeds PS-5
Steal/build: J-96 mints a case node ONLY on a proven fix; agent-s's S2 KnowledgeBase has the richer retrieve/
  distill/fuse LOOP (narrative + episodic tiers, embedding retrieval, fusion into the next plan) but saves EVERY
  run's summary with NO success gate — the exact unverified-save J-96 exists to close. ADDS: port the agent-s
  retrieve/distill/fuse loop onto J-96, but GATE the save behind J-96's mint-on-proof (execution-grounded) filter;
  add a bBoN behavior-narrative comparative judge run OVER JOURNAL ROWS (we have ground-truth action/effect deltas;
  agent-s needs screenshots) as an I-evidence-ranked-search tie-breaker (receipts pick first, narrative breaks ties).
  An upgrade + citable prior art, not a copy.
COMPOSES-WITH: J-96 verified case graph CORE (EXTEND the reuse loop — re-implements nothing), K-18 case-reuse
  economics bench (the PS-5 number this feeds), K-19 case-reuse fitness discipline (the doctrine it obeys), I
  evidence-ranked search (the bBoN judge slots in). Do NOT copy agent-s's ungated commit.
Integration: crates/orchestrator (J-96's case graph) + retrieve/distill/fuse fns gated by the mint-on-proof filter +
  a bBoN-over-journal tie-breaker; consumed by the reuse path behind J-96's strict execution-grounded filter
  (default-off). frontend_pair: rides J-96's 'memory' CanvasMode node chrome.
observable_effect: run a proven fix then a recurrence; the distill/fuse loop retrieves the case, the save is GATED
  (an unproven run's summary is NOT saved), the bBoN judge ranks candidates from journal rows; a reuse-hit field
  lands on the J-96 case row + K-18's recurrence-hit-rate reflects it; NEGATIVE: a case node saved from an unproven
  run, or a bBoN judgment sourced from screenshots instead of journal rows, FAILS.

### P-16 — gated install catalog
Track COMPUTER-USE-STEAL | lever capability-on-demand-gated | grounding MODERATE (openclaw ClawHub install-catalog +
  open-computer-use tools_config verified; the fail-closed inversion is ELAI's) | nature EXTENSION | effort M | WAVE CU-3 | depends P-9,P-10 | PROOF_SPINE none (distribution), owner-acked 2026-07-11
Steal/build: the field norm is install-on-demand (agent-zero ungated apt-get, open-computer-use tools_config
  catalog, openclaw installPolicy-gated registry); P-9's pack lane is the gated answer but nothing specs the CATALOG
  that makes capability-on-demand FEEL instant. ADDS a named install catalog ({tool, source, mirror, alias,
  license_tag} rows) + an installPolicy admission BEHIND the existing family (L-12/skillspector/N-6/
  hidden-until-verified), so "download blender/photoshop on demand" resolves to a P-9 HEADLESS code-pack connector
  (blender --background --python) — deterministic, journalable, replayable — with a P-12 computer-use GUI stage the
  LAST resort, never the mechanism. The owner's "workflows that install programs" idea, done fail-closed.
COMPOSES-WITH: P-9 connector/stage-pack format (the install target is a pack — EXTEND), P-10 credential vault (the
  auth plane), L-12 skillspector admission (the gate — reuse, do NOT add a second scanner), N-6 provenance lattice,
  P-12 operator lane (the GUI-only last resort). Do NOT copy openclaw's advisory regex scanner (V5: no teeth).
Integration: crates/orchestrator install catalog table + installPolicy check on the L-12 admission path; `elai
  connector install <tool>` CLI resolving a catalog row to a P-9 pack. frontend_pair: catalog + admission verdict in
  the connector surface (Tier-2).
observable_effect: install a catalog tool with a clean license_tag then one flagged by installPolicy; one
  install_catalog_admit row {tool, source, license_tag, verdict} per attempt; the clean one mounts a P-9 headless
  pack, the flagged one REFUSES (no install); NEGATIVE: an install that bypasses the L-12 admission family, or a
  GUI-driving install when a headless CLI exists, FAILS.

### P-23 — journaled co-browse lane (CU-4 addendum)
Track COMPUTER-USE-STEAL | lever co-browse-with-receipts | grounding STRONG on the comparator (ClaudeDevs post
  2075635283211772279 fetched verbatim 2026-07-11 + 9to5mac + Tauri multiwebview status searched), MODERATE on ELAI
  anchors (PLAN-asserted, W-7) | nature EXTENSION (H-15 + E-14) | effort M | WAVE CU-4 | depends P-13 | PROOF_SPINE none (product), owner-commissioned 2026-07-11
Steal/build: Claude Code desktop shipped an in-app browser (2026-07-10; sandboxed clean profile, model clicks any
  site, zero evidence discipline) — eval ENTRY 14's one design-level hole: ELAI has no general co-browse surface.
  ADDS a canvas browse node over the E-14 engine: Tier-1 read-only extraction ("pull up docs", taint-tagged, H-15
  deixis); Tier-2 co-browse via CDP screencast where the pane is DUMB GLASS and one InputProxy (private engine input
  handle — bypass cannot compile) journals every actor-tagged nav/click/keystroke BEFORE dispatch; typed session
  persistence (ephemeral default | P-10 vault-custody named profile); outbound typed text gated by the P-6 privacy
  lattice + domain-first-seen warning; model points by P-13 element id, coordinate fallback under P-12 frame-binding.
  Native Tauri multiwebview EXCLUDED as mechanism (unstable cargo flag, issues #10131/#10420, grounded 2026-07-11);
  survives only as a latency-triplined FORK with InputProxy still interposed.
COMPOSES-WITH: E-14 (the engine — never a second browser substrate; threaded contract if E-14 unexecuted at edit),
  H-13/H-15 (browse node view_type + deixis — EXTEND), P-13 (pointing), P-14 (a co-browse exploration crystallizes
  to an RRA-bound replay script), P-10 (profile custody), P-6 (egress), G-1/G-2 (restate none), J front-door router.
Integration: crates/orchestrator/src/cobrowse/{mod.rs,input_proxy.rs}; `elai browse <url>` CLI + front-door browse
  lane + H-13 canvas dispatcher, all wired same-changeset. frontend_pair: browse node (screencast/extraction render
  + actor-colored input trail).
observable_effect: `elai browse --fixture tests/fixtures/cobrowse/docs_page.json` lands one cobrowse_session row +
  one PRIOR actor-tagged cobrowse_input row per dispatched input (ordering checked off-agent against the engine's
  CDP log); an ephemeral session leaves zero profile bytes; NEGATIVE: an input reaching the page with no prior row,
  a model coordinate action on a stale frame hash, or a persistent profile outside vault custody, FAILS.

### P-24 — mobile design/test lane (CU-5 addendum; the deferred H-14 smartphone slice)
Track COMPUTER-USE-STEAL | lever mobile-as-a-dimension-not-a-subsystem | grounding STRONG on the external legs
  (Expo web production-grade SDK 54+ via Metro/react-native-web; Maestro single-binary YAML flows on emulator/sim —
  both searched 2026-07-11), MODERATE on ELAI anchors (PLAN-asserted, W-7) | nature EXTENSION (H-14 + P-12/P-13/P-14
  + E-14) | effort L | WAVE CU-5 | depends H-14,P-12,P-13,P-14,P-16 | PROOF_SPINE none (product), owner-commissioned 2026-07-11; Rung-3 crystallize transitively feeds PS-5 via P-14 (declared)
Steal/build: H-14 defers RN/Expo explicitly (H-14:7, :184 — and the current gig's locked stack IS RN); the corpus
  has zero other mobile coverage while 4D/4E already claims a ConsumerMobileApp product type it cannot preview,
  click-edit, or test. ADDS three rungs, cheapest first, each a DIMENSION on an existing lane: Rung 1 a typed
  DeviceProfile catalog (DATA rows) + a device_frame render variant on H-14 preview / P-23 browse nodes + an E-14
  device MATRIX with per-profile SSIM goldens (CDP metrics emulation, in-repo capture precedent); Rung 2 Expo-web
  design mode — an Expo dev server is a frameable origin so H-14's proxy + data-oid tagging + recast writeback
  extend to RNW output unchanged, click resolves RN component AST ranges; Rung 3 the native emulator as a P-12
  sandbox backend (pinned-snapshot boot, hash in the RRA pin) + mobile_axtree as a new P-13 source tag (uiautomator/
  XCUITest a11y; pixel tiers only on empty trees, J-1 unchanged) + P-14 crystallizing a Maestro YAML flow that is
  ACTUATION-ONLY (replay verdicts come from the same off-agent SSB/screenshot evidence leg as the minting run),
  provisioned via the P-16 catalog. Every result row carries render_source (web_viewport|rnw_web|native_emulator);
  a lower-rung pass NEVER discharges a higher-rung gate — "mobile tested" becomes a typed claim. Physical devices
  are ABORT-class (not disposable, not snapshot-pinnable); RN native click-to-edit (Fiber inspector) stays a
  registered follow-up inside Rung 3's FORK.
COMPOSES-WITH: H-14 (B0 proxy + tagging + writeback — EXTEND, cannot front-run it), E-14 (device matrix, never a
  second runner), P-12 (emulator = another sandboxed GUI), P-13 (mobile_axtree tag), P-14 (Maestro crystallize
  target), P-16 (emulator/SDK/Maestro provisioning), P-23 (device-frame on the browse node), 4I (capture-evaluate
  doctrine), 4D/4E + B-42 (the mobile product type + store artifacts this finally makes testable).
Integration: crates/orchestrator/src/mobile/{device_profile.rs,emulator_backend.rs}; `elai e2e run --device` +
  `elai operator run --backend emulator` CLI; device_profile field on task_runner stages; H-13 device_frame render
  variant. frontend_pair: phone-bezel node variant + per-profile pass/fail chips; iOS-without-macOS renders as the
  honest-gap affordance, never a silent skip.
observable_effect: `elai e2e run --device pixel8 --fixture tests/fixtures/mobile/journey_web.json` emits per-profile
  axtree_snapshot rows tagged device_profile + render_source; a Rung-3 run lands one emulator_session row
  {snapshot_hash, backend} and crystallizes an RRA-bound Maestro flow; NEGATIVE: a rnw_web pass presented against a
  native gate, an unpinned emulator image, or a VLM-sourced mobile count, FAILS.

==== TRACK OPENCLAW-STEAL ====
Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (full-surface AS-SPECIFIED comparison; 20-item
steal list S1..S20 + two opus deep source reads, corrections C1..C4 / confirmations V1..V5 / net-new N1..N2).
Premise: openclaw is the real peer (ships O/Q execution semantics today) but has no seeded determinism, no
replay-verify, no runtime off-agent gate, no claim grounding, and scopes prompt-injection OUT while ingesting ~30
untrusted channels — so the steal is durable-execution / ops / failover MECHANICS the corpus specs thinly; the trust
spine stays ELAI's. STEAL DESIGNS, NEVER CODE (openclaw is agent-generated at high velocity; every mechanism was
source-verified before it entered a unit here). ELAI anchors index-derived/PLAN-asserted (W-7).

### P-17 — provider failover + honest routing economics [FEEDS PS-4]
Track OPENCLAW-STEAL | lever failover-semantics-package | grounding STRONG (openclaw failover is its most
  battle-tested subsystem, source-verified; J-28/I-50 PLAN-asserted) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE feeds PS-4
Steal/build: ELAI's rotation-pool spec is thinner than openclaw's shipped failover. ADDS the whole failure-semantics
  package into J's free-API rotation pool + I-50 auth broker: (a) a failure-class cooldown taxonomy with separate
  billing/auth/rate-limit lanes (rate 30s->1m->5m; billing 5h->24h; auth 10-60m), (b) MODEL-scoped vs profile-scoped
  cooldowns (a sibling model still tries), (c) session-stickiness-for-cache-warmth as an explicit invariant
  (rotation is a cache-invalidation event, journaled as one — composes H cache-stable layout), (d)
  persist-override-BEFORE-retry (close the live-switch race), (e) strict-explicit-selection (a user's explicit model
  choice is NEVER silently substituted — matches Q-4 swap-restart's spirit: substitution is a recorded event). PLUS
  S4: never-merge billed-truth-with-estimate as typed end-to-end fields (any surface shows which it is showing).
COMPOSES-WITH: J-28 token ledger (EXTEND the UsageTurnEvent with typed billed/estimate fields — do NOT merge them),
  J free-API rotation pool + I-50 subscription auth broker (the failover consumers), B-37 runtime adapters (the
  fallback chains), Q-4 swap-restart (substitution-as-recorded-event), K-2 routing-economics bench (the PS-4 number
  honest failover feeds). 
Integration: crates/providers failover state machine (cooldown taxonomy + model/profile scope + stickiness) +
  J-28 typed billed/estimate fields; consumed by the provider dispatch path. frontend_pair: failover + a
  billed-vs-estimate label on the usage surface (RuntimePanel/CloudPanel, W-4).
observable_effect: force a billing failure then a rate-limit on one provider; a provider_failover_event row per
  event with the correct lane + cooldown; a sibling model on the same provider still tries; the usage surface shows
  billed vs estimate as distinct typed fields; NEGATIVE: a billing failure that cools the whole provider (not just
  the billed lane), a silent substitution of a user's explicit model choice, or a merged billed/estimate number, FAILS.

### P-18 — durable taskspec exec + approval drift-binding
Track OPENCLAW-STEAL | lever durable-runs-anti-toctou | grounding STRONG for the verified half (Task-Flow
  revision-counter + approval drift-binding source-verified V-grade); WEAK-until-RECON for the resume-token step-skip
  (C1 unverified-external) | nature EXTENSION | effort M-L | WAVE OC-2 | depends P-11 | PROOF_SPINE none (product), owner-acked 2026-07-11
Steal/build: O-2/P-11 have the trust semantics but not the boring durable-execution correctness. ADDS to the
  TaskSpec run store: revision-counter optimistic concurrency (a stale expectedRevision -> conflict, never clobber;
  the VERIFIED mechanism, S6/ENTRY13) + sticky persisted cancel intent surviving restart + retry-ladders-per-
  failure-class + watchdog/preflight phases + the commitment due-time clamp (an inferred follow-up may never fire in
  the tick it was inferred, S19). PLUS approval drift-binding (anti-TOCTOU, S1): every human/autonomous approval
  BINDS a canonical hashed run-plan (cwd/argv/env/pinned-executable) + concrete operand fingerprints; execution
  re-validates; ANY drift = deny. The lobster "approve+resume never re-runs prior steps" claim is UNVERIFIED-external
  (C1: un-vendored type-stub; the in-repo wrapper force-errors needs_input) — carried as RECON_NEEDED and
  re-derived as a REAL subprocess-checkpoint model (SIGKILL + pipe backpressure + a step-checkpoint store), stronger
  than their cooperative enforcement.
COMPOSES-WITH: O-2 TaskSpec substrate + O-4 canvas (EXTEND the run store), P-11 workflow-from-goal confirm gate
  (drift-binding hardens MOVE 3), 3A per-transition gates + 4G autonomous gate (every approval binds a plan), G
  blast-radius gates (the drift check composes). 
Integration: crates/orchestrator TaskSpec run store (revision counter + sticky cancel + retry ladder) + an
  approval-plan hasher/validator on the 3A/4G/P-11 approval seams; `elai workflow resume|cancel`. frontend_pair:
  run-revision + drift-reject rows in the workflow view.
observable_effect: two concurrent writes to one TaskSpec run (one on a stale revision) + an approval whose operand
  is swapped after grant; one taskspec_run_revision conflict (the stale write refused, no clobber) + one
  approval_drift_reject on the swapped operand; a cancel survives a simulated restart; NEGATIVE: a last-writer-wins
  clobber, an approval that executes a drifted plan, or a commitment firing in its own inference tick, FAILS.

### P-19 — config/ops lifecycle + elai doctor + verified backups [NEW area]
Track OPENCLAW-STEAL | lever recoverable-config-and-state | grounding STRONG (openclaw's most underrated subsystem,
  source-verified; ELAI's thinnest area) | nature NEW-area (composition-heavy) | effort M-L | WAVE OC-2 | PROOF_SPINE none (ops), owner-acked 2026-07-11
Steal/build: ELAI's thinnest area vs the trust story it sells (a trust product that corrupts its own config or can't
  restore state undermines the story cheaply). ADDS: (a) a fail-closed-default-object config posture (policy objects
  fail closed even when ABSENT, cf openclaw ssrfPolicy) + last-known-good + rejected-write preservation
  (.rejected.<ts>) + shrink/clobber guards (>50% shrink rejected) + atomic writes, into J's config-ingress gate
  (which today only admits — it should also make bad writes recoverable + journaled); (b) an `elai doctor`
  detect/repair/lint triple with STABLE exit codes for CI (ELAI already worships deterministic exit codes) +
  post-upgrade compat probes; (c) VACUUM-INTO WAL-safe verified SSB/session backups + manifest verify (the journal
  IS the product — it deserves a verified backup); (d) a restart continuation queue (one in-flight turn survives a
  restart); (e) the compaction split-rule (never separate a tool-call from its result) + successor-transcript
  (never rewrite in place) as stated invariants. INVERT openclaw's C2 bug: env is ALLOWLISTED, never full-copied;
  adopt their cwd path-jail (case-fold-before-compare on win32) verbatim.
COMPOSES-WITH: J config-ingress admission gate (EXTEND to recoverable), M signed-release/support-bundle (the doctor
  + backup live here), 3H session handoff + H context lifecycle (the compaction split-rule + successor-transcript —
  make explicit), Q ops hardening. 
Integration: crates/orchestrator config lifecycle (LKG + rejected-write + guards) + new crates/cli doctor_cmd.rs
  (detect/repair/lint + exit codes) + SSB VACUUM-INTO backup + restart continuation queue. frontend_pair: N/A with
  justification (CLI + ops surface; doctor output is structured text, W-6 N/A). Justified.
observable_effect: write a config that shrinks the file >50% + run `elai doctor lint` + kill mid-turn; the bad write
  is REJECTED and preserved as .rejected.<ts> with a config_write_rejected row + doctor lint returns a stable
  nonzero exit + structured doctor_finding rows + the in-flight turn resumes once after restart; NEGATIVE: a >50%
  shrink that overwrites, a doctor exit code that varies run-to-run, or a compaction that splits a tool-call from its
  result, FAILS.

### P-20 — context/memory lifecycle hardening
Track OPENCLAW-STEAL | lever no-context-lost-uncached | grounding STRONG (openclaw flush + on-demand index
  source-verified; H anchors PLAN-asserted) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE none
Steal/build: ELAI's context spec is strong on invariants, thin on lifecycle mechanics. ADDS: (a) a pre-compaction
  memory-flush as an ORDERED journaled step (a silent NO_REPLY-equivalent turn writes durable notes BEFORE
  compaction, once per cycle) — the invariant is "no compaction may destroy context that was never offered to
  durable memory"; in-session, distinct from H-51's cross-session sleep-time handoff; (b) a versioned on-demand
  skill/rules index (a compact prompt index carrying name/description/path + a sha256 VERSION marker ~24 tokens/skill;
  the model reads the file only when needed, re-reads on version change; budget compact-degrade) — ONE lazy-load
  mechanism for skills + rules + MCP (composes H MCP defer-loading); (c) Dreaming's promotion gates
  (score/recall/diversity) as prior art for H memory governance; (d) the per-provider cache-control mapping table
  (Anthropic cache_control 5m/1h; OpenAI prompt_cache_key; Gemini cachedContents — their tested knowledge of what
  each provider actually honors) into H's caching implementation.
COMPOSES-WITH: H-50/H-51 context lifecycle (EXTEND with the flush step), H two-tier user memory (the flush target),
  L familiarity surface + H MCP defer-loading (the on-demand index — one mechanism), H cache-stable layout (the
  per-provider table). Do NOT add a second memory store.
Integration: crates/orchestrator context lifecycle (flush hook on the compaction path) + a versioned skill/rules
  index loader + a per-provider cache-control table. frontend_pair: N/A with justification (internal context
  machinery; no user surface beyond existing cost/memory panels, W-6 N/A). Justified.
observable_effect: drive a session to the compaction threshold; a memory_flush_event row lands BEFORE the
  compaction row (ordered), carrying the durable notes; the skill index shows only compact entries until a skill is
  needed then the file is read once (trace); NEGATIVE: a compaction that fires before its flush, or a full skill
  body loaded into the prompt before the skill is invoked, FAILS.

### P-21 — evidence / gate-credibility hardening [FEEDS PS-6]
Track OPENCLAW-STEAL | lever prove-the-gate-still-bites | grounding STRONG (verify-the-verifier + structural
  redaction VERIFIED-IN-SOURCE V1/V2; Q-6 PLAN-asserted) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE feeds PS-6
Steal/build: openclaw's fail-closed INSTINCTS are real + test-covered and independently converge on ELAI doctrine —
  steal them as reference implementations into the gate corpus. ADDS: (a) verify-the-verifier seeded
  negative-control canaries (build known-BAD artifacts, run them through the SAME production detectors; a missed
  canary hard-BLOCKS the whole gate, exitCode=1 — V1, ENTRY13 amendment) into Q-6's evidence-corruption corpus;
  (b) missing-artifact=FAIL with an in-manifest explicit escape (a missingVerdict may ONLY be environment-blocked or
  optional-gap — N1, structurally ~= the WIRE-DARK allowlist); (c) skipBackfill cross-lane dependency (a lane with
  skipped rows clears ONLY if a NAMED sibling passed — N2, ~= wiring-R10 threaded-required-input); (d) STRUCTURAL
  redaction (the audit/export row is a CLOSED metadata struct so content has nowhere to leak; hash-don't-store ids;
  whitelist-or-"unknown" fields — V2), keeping the shareable metadata projection STRICTLY separate BY TYPE from the
  content-carrying debug bundle (V3 — the exact conflation trap); (e) posture labels on smoke evidence
  (user-path / direct-gateway / native-approval) so R10 evidence can't silently over-claim (S9). COPY unknown=FAIL,
  do NOT copy openclaw's default that lets a classified product-bug pass a green report (C3).
COMPOSES-WITH: Q-6 gate-manifest evidence-corruption corpus (the canaries + kill-rate — EXTEND, the PS-6 number),
  wiring-enforcement R10 (posture labels + skipBackfill), G-9 redaction gate (STRUCTURAL redaction), M fleet
  evidence rollup (the metadata-only projection). 
Integration: crates/orchestrator Q-6 corpus (add the verifier-canary lane + the manifest missing-escape) + a
  posture-label field on the R10 smoke evidence + a closed-metadata audit-export struct (G-9). frontend_pair: rides
  the existing gate/evidence surface (W-4).
observable_effect: run the gate corpus with a canary detector deliberately broken; the self-test lane detects the
  miss and the WHOLE gate goes to fail (exit 1); a smoke with no posture label FAILS; the audit-export struct has no
  content field to leak into; NEGATIVE: a missed canary that still passes the gate, a classified failure passing a
  green report by default, or content appearing in the metadata-only export, FAILS.

### P-22 — control-plane / channel hardening
Track OPENCLAW-STEAL | lever fenced-returns-signed-ingress | grounding STRONG (openclaw gateway auth mechanics
  source-verified; ELAI anchors PLAN-asserted) | nature EXTENSION | effort M-L | WAVE OC-3 | PROOF_SPINE none
Steal/build: openclaw's control-plane + multi-agent + channel policy encode years of edge cases. ADDS: (a)
  child-output-as-fenced-evidence (a subagent return is DATA structurally unable to carry instructions — composes
  G-1: a subagent return is a taint source, period) + depth-scoped tool-strip defaults (subagents lose
  gateway/cron/messaging; leaves lose spawn) for 3C; (b) the 9-step deterministic channel-binding precedence
  (most-specific-wins; the model never picks the channel) + pairing-code UX (expiring short codes, pending caps) +
  dmScope session-isolation options for P-10 ingress adapters + the front-door router + M enrollment; (c)
  challenge-signed device pairing (platform-bound signature payloads, stable auth error codes) + fail-closed
  unknown-event broadcast gating (an event family the reader doesn't recognize is WITHHELD, not leaked) + mandatory
  idempotency keys on side-effecting RPC, for the J control plane + M seats; (d) the trusted-policy hook tier ordered
  BEFORE third-party hooks (ELAI's frozen gates occupy an ordered tier no plugin can preempt) + manifest-validated-
  without-execution, for 4C admission.
COMPOSES-WITH: 3C agent system + BSP (the fenced returns + tool-strip), G-1 taint (subagent-return-as-source),
  J control plane + P-10 ingress adapters + M seats (the auth mechanics), 4C plugins (the hook tier + manifest
  validation). Do NOT fork a second provenance scale (N-2/A-1).
Integration: crates/orchestrator 3C (fenced returns + depth tool-strip) + the J control-plane auth (signed pairing +
  unknown-event gating + idempotency) + P-10 channel-binding precedence + 4C hook tier. frontend_pair: enrollment +
  pairing UX on the control surface (Tier-2, W-4).
observable_effect: a subagent returns text shaped like an instruction + an unknown event family hits the broadcast +
  a mutating RPC replays its idempotency key; the subagent return lands as a fenced taint-source row (the instruction
  is inert), the unknown event is WITHHELD not leaked, the replayed RPC is deduped; NEGATIVE: a subagent return that
  steers the parent, an unrecognized event family that leaks to the reader, or a double-applied mutating RPC, FAILS.

## Track COMPUTER-USE-STEAL + OPENCLAW-STEAL cross-cutting (what makes them different)
Both tracks are STEAL tracks (paper-to-plan Phase-4 -> Phase-5) and inherit P's whole discipline: PROOF_SPINE
declared per unit (P-14/P-15 PS-5, P-17 PS-4, P-21 PS-6; the rest none, owner-acked 2026-07-11), the A-1/N-2 no-
second-lattice/scheduler rule (P-13/P-22 ride the existing taint + provenance lattice), STEAL DESIGNS-NEVER-CODE for
every external mechanism (comparators source-verified in the two eval docs; every openclaw claim carries its
C1..C4/V1..V5 grade). The unifying thesis both tracks confirm: the six computer-use repos and openclaw all LACK the
one property ELAI specs — off-agent, journaled, replayable, fail-closed evidence — so ELAI does not need to beat them
on capability breadth; it CONTAINS them (the I-40 `elai wrap` inversion: a capability winner becomes a vendored
gated backend, P-12/P-16). Deep red-team is DEFERRED to each unit's execution pre-flight (the legacy-plan
refine-before-execute path, as P-9..P-11); the REDTEAM block in each detail file records the surviving-attack seed.

==== TRACK CREATIVITY-FALSIFIER ====

### P-25 — exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0)
Track CREATIVITY-FALSIFIER | levers deterministic-exogenous-injection + grounded-loop-revision | grounding MEASURED
  (pilot P-A 3.5x anchors) for the injection lever, HYPOTHESIS for both creative-yield effects | nature NEW(falsifier
  bench, harness-only) | effort M | authored 2026-07-14, user-directed successor to Q-10 B-CREATIVITY-0 REFUTED
  (b0v3); FAMILY B = the grounding-as-exit doctrine instantiated at creativity grain | NOT spine-gated (evidence run)
Steal/build: the track's two surviving untested lever families on ONE frozen corpus — K=30 open-world real ELAI
  improvement fixtures with hidden deterministic micro-executors (B-CREATIVITY-1 corpus design, never built) — six
  arms, two independently-preregistered joint gates, no cherry-picking between them (multiplicity declared).
  FAMILY A (injection): the single direct champion call conditioned on k=3 deterministically-retrieved MODEL-FREE
  target-relevant verified artifacts (elai.map anchor slice + measured bench/journal row + VerifiedCitation) vs
  DIRECT / cross-subsystem-stratified EXO_SHUFFLED / PAD_MATCHED; HARD 1.3x cost cap (the 8.8x killer preregistered
  away); zero preprocessing model calls IS the mechanism. FAMILY B (revision): TRIAGE_REVISE — generate, run the
  REAL deterministic public triage legs (demarcation + anchor + composition; ledger excluded; exo card grammar
  emits the judged fields, KEY_ALIASES-reconciled), feed per-candidate kill reasons into ONE revision call — vs
  SHUFFLED_REVISE (identical template, kill reasons permuted off their true candidates); HARD 2.5x cap +
  cost-normalized P1 non-inferiority (a win bought by double spend FAILS). Co-primaries per family: execution-
  validated mechanism yield + best target value delta, paired bootstrap CI95. Materially different from every
  refuted mechanism (no packetizer/analogy/archive-conditioning/operator-rotation/probes; B-3 tested archive
  cycles, §6 tested parallel context-blind calls — neither tested sequential evidence-coupled revision);
  rejection-ledger clearance recorded in the file. Red-teamed twice (authoring + delta), all BLOCKs patched.
COMPOSES-WITH: Q-10 Tier-0 instrument modules (import, never grow the 1000-line runner), conjecture_triage.py
  kill legs (the FAMILY B evidence source), research_creativity analysis/verifier, elai.map, E-48 VerifiedCitation
  rows (read-only substrate), grounding-as-exit doctrine note (the FAMILY B design source).
Integration: .claude/tools exo_retrieval.py + exo_injection_ab.py (six-arm, two-call revision lifecycle) + frozen
  triage-render template + fixture corpus; NOTHING in .elai_cc.
observable_effect: sealed evidence dir + independent checker exit 0 (incl. revision-input byte-check); a SUPPORTED
  family -> its own separate adoption unit (FAMILY A: improvement-conjecture stage-1 exo leg + paper-to-plan seam;
  FAMILY B: a triage-feedback revision leg); BOTH NULL/REFUTED -> the creativity line CLOSES entirely; NEGATIVE:
  a retrieval- or triage-time model call, hidden-side data in a revision input, a second revision round, a shuffled
  arm matching treatment relevance, or a cost row past its family cap without G3 firing must FAIL.
