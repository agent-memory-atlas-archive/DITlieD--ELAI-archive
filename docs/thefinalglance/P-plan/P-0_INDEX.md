# P-Plan Index — the PERSONAL-SOFTWARE INCARNATION side, wiring-enforced (25 sessions; Tracks PERSONAL-INCARNATION + COMMAND-CENTER + COMPUTER-USE-STEAL + OPENCLAW-STEAL + CREATIVITY-FALSIFIER authored)
Date: 2026-07-07 (authored from the spec-eval design digest; owner-acked PROOF_SPINE: none). Third/fourth waves 2026-07-11.

Source synthesis: workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md (the AS-SPECIFIED spec-eval of
  "ELAI as the software incarnation of the bespoke-tools era" — the r/ClaudeAI bespoke-tools thread's six demands
  T1..T6 mapped against the corpus, eight gaps GAP-1..GAP-8, an adversarial pass A-1..A-7, a sequencing sketch).
  The digest's eight gaps GAP-1..GAP-8 are 1:1 with P-1..P-8 below. Its adversarial pass A-1..A-7 and its
  substrate-per-gap lists feed each unit's REDTEAM + stub_ban.
Cross-source: reddit r/ClaudeAI thread archive (bespoke tools, 1uopekl, 103 comments) — the six demands are
  quoted to specific commenters in each unit's Grounding, never a strawman. External anchor: Ink & Switch
  malleable-software essay (inkandswitch.com/essay/malleable-software, June 2025) + geoffreylitt.com — the named,
  active research field P aims at.

SECOND WAVE — COMMAND-CENTER GAP UNITS (P-9..P-11, authored 2026-07-08/09): a distinct source from the P-1..P-8
  personal-incarnation set. Source: workflow/reasoning/elai-command-center-eval-2026-07-08.md (the AS-SPECIFIED
  spec-eval of "ELAI as the single command center" + the schema-design discussion + the connector/node economics). Its
  §4 ranked changes 1/2/3 = P-9/P-10/P-11. All three PROOF_SPINE: none (distribution), owner PSR-2-acked 2026-07-09;
  all gate-PASS (plan_gate_check exit 0); all execute AFTER the proof spine (A-7), red-team DEFERRED to their
  execution pre-flight (the legacy-plan refine-before-execute path).
  - P-9 connector-pack admission format (GAP 1, ranked #1): M-6 own-the-FORMAT applied to CAPABILITY — a signed,
    versioned declarative connector/stage-pack (ONE generic http_call kind parameterized by a DATA spec) + a code
    lane (B-37), mounted through the EXISTING admission family (L-12/skillspector/J-102/N-6). Converts the closed
    vocabulary from ceiling into moat. KEYSTONE of the second wave.
  - P-10 ingress adapters + credential vault (GAP 2): an IngressAdapter family normalizing all four external shapes
    (pull/push/streaming/other) to a taint-tagged (G-1) journaled SsbEvent so the O-2 trigger enum does NOT grow,
    + a first-class CredentialVault (the auth plane P-9's http_call needs; secrets never journaled/exported).
  - P-11 elai workflow from-goal (GAP 3): intent -> a CLOSED-enum-constrained proposed TaskSpec graph on the O-4
    canvas, human-confirmed before it runs, with an L-14 RRA receipt. The n8n-user-converting demo (H-13 + O-2).
  Sequencing within the wave: P-9 (vocabulary) -> P-10 (auth/ingress substrate) -> P-11 (front door). GAP 4 (P-8
  merge-kernel spike) + GAP 5 (distribution sequencing) from the same doc are NOT new files (P-8 exists; GAP 5 is
  the discipline already written down).

THIRD WAVE — COMPUTER-USE-STEAL (P-12..P-16, authored 2026-07-11): a distinct source from the personal-incarnation
  and command-center sets. Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (the AS-SPECIFIED
  spec-eval of "is ELAI's canvas + e2e-testing surface better than agent-zero / openclaw / omniparser /
  open-computer-use / trycua-cua / simular agent-s, and what do we steal", 13 entries incl a two-opus source read
  of OmniParser + agent-s + cua). Its PRIORITIZED STEAL LIST P1..P8 collapses (after de-dup with the openclaw pass)
  into five vertical slices. The central finding: on the axis ELAI actually claims (journaled, replayable,
  fail-closed, typed computer control with receipts) NONE of the six matches the spec, but the six expose ONE genuine
  design-scope gap regardless of timing — ELAI specs browser E2E (E-14) + media validation (4I) but has NO general
  desktop/GUI computer-use lane, and no specced instance of "an agentic run mints a deterministic replayable artifact".
  - P-12 desktop/GUI operator lane (the honest gap; NEW): cua sandbox as vendored substrate + OmniParser Tier-3
    pixel fallback + openclaw frameId/display fail-closed binding + per-action bounded screen-recording as off-agent
    evidence + crosshair/zoom evidence crops, ALL under ELAI's gates (cua's own in-loop safety is a stubbed TODO).
  - P-13 perception provenance + Set-of-Marks ID-indirection (EXT E-14 + J-1/J-8/J-77): the VLM picks an integer id
    off a tagged element list, never coordinates (OmniParser's measured 70.5->93.8% grounding lift); every element
    carries a source tag (axtree_exact | ocr_text | vlm_caption | pixel_yolo) as the trust boundary; Tier-3 pixel
    fires only when AXTree is empty/provably incomplete (counts never from a VLM, J-1 doctrine unchanged).
  - P-14 explore-then-crystallize record->replay lane (EXT E-14 + O-2): an agentic exploration run emits a
    deterministic Playwright/pyautogui script AS an O-2 TaskSpec stage with a G-2 RRA receipt binding the script to
    the exploration run that minted it; replay = the cheap execution mode. Prior art: cua replay_env, agent-zero
    scripts/ convention (openclaw jsonl-replay is INPUT-through-a-mock, not a real record->re-score; C4). FEEDS PS-5.
  - P-15 verified case-graph hardening (EXT J-96 + K-18/K-19): port agent-s S2 KnowledgeBase retrieve/distill/fuse,
    but GATE the save behind J-96's mint-on-proof filter (agent-s saves EVERY run unverified); add a bBoN
    behavior-narrative comparative judge run OVER JOURNAL ROWS (ground-truth action/effect deltas) as an
    evidence-ranked-search tie-breaker. FEEDS PS-5 (K-18 case-reuse compounding is the bench).
  - P-16 gated install catalog (EXT P-9 + P-10 + L-12): named {tool, source, mirror, alias, license_tag} rows +
    an installPolicy behind the EXISTING admission family (skillspector/L-12/N-6/hidden-until-verified); the
    sanctioned "download blender/photoshop on demand" front door is a P-9 code-pack connector wrapping the app's
    headless CLI (blender --background --python), with the P-12 computer-use GUI stage the LAST resort, never the
    mechanism (settles the owner's "workflows that install programs" idea the fail-closed way).
  CU-4 ADDENDUM (P-23, authored 2026-07-11, same source doc ENTRY 14): the Claude Code desktop in-app browser
  (ClaudeDevs post 2075635283211772279, 2026-07-10) exposed the one design-level hole the six repos did not — no
  general co-browse surface (E-14 is test-scoped, canvas RESEARCH mode research-scoped, P-12 a last-resort operator
  lane). P-23 closes it the ELAI way.
  - P-23 journaled co-browse lane (EXT H-15 + E-14): a canvas browse node over the E-14 engine — Tier-1 read-only
    extraction for "pull up docs"; Tier-2 co-browse via CDP screencast where the pane is DUMB GLASS and ONE
    InputProxy (private engine input handle) journals every actor-tagged input BEFORE dispatch, so an unjournaled
    input path cannot compile; typed session persistence (ephemeral default | vault-custody named profile, P-10);
    outbound typed text gated by the P-6 privacy lattice; browse content G-1-tainted (never authorizes a mutating
    lane); native Tauri multiwebview EXCLUDED as mechanism (unstable flag, grounded) and survives only as a
    latency-triplined FORK. The pitch: the only co-browse surface with receipts.
  CU-5 ADDENDUM (P-24, authored 2026-07-11, same session): the H-14 smartphone deferral ("RN/Expo deferred to a
  later separately-scoped slice", H-14:7; the current gig's locked stack IS RN, H-14:184) closed as three honest
  rungs on the P computer-use spine — mobile as a typed DIMENSION on existing lanes, never a parallel subsystem.
  - P-24 mobile design/test lane (EXT H-14 + P-12/P-13/P-14 + E-14): Rung 1 DeviceProfile catalog (DATA) +
    device_frame render variant on H-14 preview / P-23 browse nodes + an E-14 device MATRIX (CDP metrics emulation,
    in-repo precedent); Rung 2 Expo-web design mode (Metro/RNW is a frameable origin — H-14 proxy + data-oid tagging
    + recast writeback unchanged; render_source=rnw_web, never claimed native); Rung 3 the native emulator as a P-12
    sandbox backend (pinned-snapshot boot hash in the RRA pin, mobile_axtree as a P-13 source tag, P-14 crystallizes
    a Maestro flow that is ACTUATION-ONLY — replay verdicts come from the off-agent evidence leg), provisioned via
    the P-16 catalog. Rung labels are typed on every row; a lower-rung pass never discharges a higher-rung gate.

FOURTH WAVE — OPENCLAW-STEAL (P-17..P-22, authored 2026-07-11): Source:
  workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (full-surface AS-SPECIFIED comparison of ELAI vs the
  openclaw personal-assistant gateway, 22,435-file reviewed source, a 20-item consolidated steal list S1..S20 + two
  opus deep source reads correcting C1..C4 / confirming V1..V5 / N1..N2). Central finding: openclaw is the REAL peer
  (independently built large parts of O/Q's execution semantics and ships them today) but has NO seeded determinism,
  no replay-verify, no runtime off-agent done-gate, no claim grounding, and EXPLICITLY scopes prompt-injection out of
  its trust model while ingesting ~30 untrusted channels — so the wedge survives fully and sharpens. The steal is
  mechanics openclaw bled for that ELAI's corpus specs thinly. Six subsystem slices:
  - P-17 provider failover + honest routing economics (EXT J-28 + I-50 + Q-4 + B-37): the failure-class cooldown
    taxonomy (rate/billing/auth lanes, model-scoped vs profile-scoped), session-stickiness-for-cache-warmth,
    persist-before-retry, strict-explicit-selection (S2) + never-merge billed-truth-with-estimate as typed
    end-to-end fields (S4). FEEDS PS-4 (K-2 routing economics needs honest failover).
  - P-18 durable TaskSpec exec + approval drift-binding (EXT O-2 + O-4 + P-11 + 3A): revision-counter optimistic
    concurrency + sticky-cancel + retry-ladders-per-failure-class + watchdog for the TaskSpec run store (S6, the
    VERIFIED half; the lobster resume-token step-skip is UNVERIFIED-external per C1, carried as RECON_NEEDED and
    re-derived as a real subprocess-checkpoint model) + approval drift-binding anti-TOCTOU (every approval binds a
    canonical hashed run-plan + operand fingerprints; drift = deny, S1) + commitment due-time clamp (S19).
  - P-19 config/ops lifecycle + `elai doctor` + verified backups (NEW area, ELAI's thinnest): fail-closed-default-object
    config posture + last-known-good/rejected-write/shrink-guard/atomic writes (S5) + doctor detect/repair/lint with
    stable exit codes + post-upgrade probes + VACUUM-INTO verified SSB/session backups + restart continuation queue
    (S13) + the compaction split-rule (never separate a tool-call from its result) + successor-transcript invariant (S14).
  - P-20 context/memory lifecycle hardening (EXT H-50/H-51 + H user-memory + L familiarity + H MCP defer-load):
    pre-compaction memory-flush as an ordered journaled step ("no compaction destroys context never offered to
    durable memory", S3) + versioned on-demand skill/rules index (sha256 markers, compact index, budget-degrade, S7)
    + Dreaming promotion gates as prior art (S16) + the per-provider cache-control mapping table (S18).
  - P-21 evidence / gate-credibility hardening (EXT Q-6 + wiring R10 + G-9 + M): verify-the-verifier seeded
    negative-control canaries that hard-block the whole gate when a canary goes undetected (S15/V1, exitCode=1) +
    missing-artifact=FAIL with an in-manifest explicit escape (N1, ~= the WIRE-DARK allowlist) + skipBackfill
    cross-lane dependency (a skip clears only if a NAMED sibling passed, N2 ~= R10 threaded-required-input) +
    STRUCTURAL redaction (the audit row is a closed metadata struct so content has nowhere to leak; keep the
    content-carrying debug bundle SEPARATE by type, V2/V3) + posture labels on smoke evidence (S9). FEEDS PS-6.
  - P-22 control-plane / multi-agent / channel hardening (EXT 3C + G-1 + J control-plane + P-10 + M + 4C):
    child-output-as-fenced-evidence + depth-scoped tool-strip defaults (a subagent return is a taint source, S10) +
    the 9-step deterministic channel-binding precedence + pairing-code UX + dmScope isolation (S11) +
    challenge-signed device pairing + fail-closed unknown-event broadcast gating + mandatory idempotency keys on
    mutating RPC (S12) + trusted-policy hook tier ordered before third-party + manifest-validated-without-execution (S20).
  Grounding for both waves: the two source reasoning docs above (self-contained spec-eval steal analyses, the
  paper-to-plan Phase-4 artifact; external comparators source-verified in the docs' own deep-read entries — OmniParser
  arXiv:2408.00203, agent-s arXiv:2410.08164/2504.00906/2510.02250, openclaw HEAD 2026-07-10 TypeScript). Every ELAI
  cross-link below is index-derived / PLAN-asserted (the executor re-greps per W-7). No separate _grounding/ map (P's
  own precedent: point at the source design doc). PROOF_SPINE: P-14/P-15 feed PS-5, P-17 feeds PS-4, P-21 feeds PS-6;
  P-12/P-13/P-16/P-18/P-19/P-20/P-22 are PROOF_SPINE: none (product-surface / distribution / ops / discipline class),
  owner-acked 2026-07-11 per PSR-2; P-23 + P-24 PROOF_SPINE: none (product), owner-commissioned 2026-07-11 (units
  explicitly requested in the co-browse gap session; P-24's Rung-3 crystallize rung transitively feeds PS-5 via
  P-14, declared per PSR-1 tier 3). All execute AFTER the proof spine (A-7); deep red-team DEFERRED to each unit's
  execution pre-flight (the legacy-plan refine-before-execute path), as with P-9..P-11.

Predecessors: B-plan .. O-plan. P COMPOSES-WITH them (GD-1); restates none. P is a PRODUCT-SURFACE letter, the
  DIRECT SUCCESSOR to O-plan (the personal RESEARCH machine); P is the personal SOFTWARE-FLEET machine. Like O it
  ships NEW user-facing features (app fleet + inter-tool bus + hand-to-friend export + living brains + non-coder
  operator surface + life-data privacy lattice + per-user operating profile + small-team world) that are ~80%
  composition of shipped subsystems, plus a small set of genuinely-new mechanisms (the AppManifest/PortSchema
  interchange format, the persona/mechanism factoring rule, the OperatingProfile asymmetry, the two-seat
  SSB-stream merge kernel).

Predecessor reading (mandatory before any P-N execution):
  workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md (the WHY: T1..T6, GAP-1..GAP-8, A-1..A-7).
  docs/thefinalglance/ELAI-FEATURE-MAP.md (the composition edges each P-N consumes — the single context load;
    do NOT re-read the legacy 12-file list).
  docs/thefinalglance/O-0_INDEX.md (P is O's successor product-surface letter; same compose-not-restate posture).
  docs/thefinalglance/F-0_INDEX.md (W-1..W-10 wiring doctrine, INHERITED verbatim).
  docs/thefinalglance/G-0_INDEX.md (GD-1..GD-5; P is a compose-not-restate letter like G/N/O).
  this file (P-0_INDEX.md), then P-SUMMARY_INDEX.md, then the target P-N detail file.

## Why P-plan exists (read this before anything else)
TRACK PERSONAL-INCARNATION (P-1..P-8). Authored 2026-07-07 from the spec-eval digest. The CENTRAL FINDING (AS-
  SPECIFIED, grounded against the feature map): ELAI as specified is NOT missing the bespoke-tools vision — it is
  missing the LAST LAYER of it. The trust spine (receipts, taint, replay, cant-lie, wiring gates) is exactly the
  thing the entire bespoke-tool world lacks and cannot get from any competitor; the reddit thread is a catalog of
  people who built magic boxes and have no way to keep them honest, alive, connected, or handed over. But the
  corpus's unit of care is a CHANGESET/TASK/SEAT, and the thesis's unit of care is a PERSON'S POPULATION OF TOOLS
  over years. The incarnation move is NOT a new engine; it is LIFTING the existing spine one grain: changeset ->
  app, task -> fleet, seat -> world, org -> household/team. Each unit is BUILD-on-shipped, not net-new:
  - P-1 makes the APP a first-class object (manifest + fleet registry + O-2 health-tick TaskSpecs + G-10 per-app
    drift canaries + gated auto-repair). Answers T2 keep-it-alive, the thread's biggest pain. KEYSTONE.
  - P-2 gives every tool typed PORTS + a world-grain event bus of provenance-carrying edges on the O-4 canvas.
    Answers T3 glue. ADDS the port-schema interchange registry.
  - P-3 factors every app MECHANISM-vs-PERSONA + a hand-to-friend export bundle re-verified on the recipient's
    seat. Answers T5 share. ADDS the export bundle (the factoring is doctrine).
  - P-4 mounts LIVING external brains (Obsidian/Notion/SQLite), not just a static vault. Answers "integrate with
    existing brains". GATED behind an Obsidian-API RECON (the digest flags it UNGROUNDED).
  - P-5 makes the trust spine scale DOWN: a graduated-ceremony toy/personal profile + a non-coder operator
    surface. Answers T1 for the non-coder. ADDS the ceremony profile on K-5/K-4.
  - P-6 adds the life-data privacy lattice (which TOOL may read which NAMESPACE) as a DIMENSION on N-6's one
    lattice — taint's out-flow mirror. Answers T6 owned-and-private.
  - P-7 adapts PROCESS not just content: a per-user OperatingProfile, asymmetric (loosen only on measured own-run
    FP evidence) so adaptation cannot become sycophancy. The three archetypes = three defaults on ONE engine.
  - P-8 is the 2-10 human shared world; the hard kernel is a defined lossless deterministic two-seat SSB-stream
    merge (a design spike first, composition second).
TRACK COMPUTER-USE-STEAL (P-12..P-16). Authored 2026-07-11 from the canvas-computer-use spec-eval. CENTRAL FINDING:
  none of the six computer-use comparators occupies ELAI's journaled+replayable+typed slot, but they expose ONE real
  design-scope gap — ELAI has NO desktop/GUI operator lane and no "agentic-run-mints-a-replayable-artifact" instance.
  P-12 fills the gap (GUI actuation under ELAI's gates, comparators as vendored substrate + operator lanes, never
  rivals — the I-40 `elai wrap` inversion applied to computer use); P-13 hardens perception (id-indirection +
  provenance); P-14 adds the record->replay crystallize lane; P-15 hardens the case graph with a gated agent-s memory
  loop; P-16 makes capability-on-demand fail-closed. Full theses + per-unit correction anchors: the THIRD-WAVE block above.
TRACK OPENCLAW-STEAL (P-17..P-22). Authored 2026-07-11 from the openclaw full-surface comparison. CENTRAL FINDING:
  the real peer ships O/Q's execution semantics today but has no seeded determinism, no replay-verify, no runtime
  off-agent gate, and scopes injection out — so the steal is the durable-execution / ops / failover MECHANICS the
  corpus specs thinly, never the trust spine (which stays ELAI's). P-17 failover economics (PS-4); P-18 durable runs +
  anti-TOCTOU approval binding; P-19 the config/ops lifecycle (ELAI's thinnest area); P-20 context/memory lifecycle;
  P-21 gate-credibility (PS-6); P-22 control-plane/channel hardening. Full theses + C1..C4/V1..V5 correction anchors:
  the FOURTH-WAVE block above.

## Status legend
PLANNED / IN PROGRESS / DONE / DEFERRED. DONE = the FULL wiring done-gate (wiring_contract_check.py --run-trigger
  PASS + production_caller_check.py/python_wire_check.py clean + the R10 integration smoke green);
  cargo-green is necessary but NOT sufficient.

================================================================================
THE WIRING DOCTRINE
================================================================================
INHERITS W-1..W-10 VERBATIM (canon: F-0_INDEX.md): real-entrypoint trigger; off-agent observable artifact;
  missing evidence = FAIL; sound-reference-not-leaf-grep; producer-without-consumer = build-break; test callers
  never satisfy reachability; frozen gate; no default-off graveyard; live baseline re-read; dashboard done-item.
INHERITS GD-1..GD-5 (canon: G-0_INDEX.md): COMPOSE-never-restate (every P-N names what it consumes and must NOT
  re-implement); GD-4 observe-only-first for any gate defaulting ON; GD-5 honest residue named in-file.
ADDS no new delta family. The A-1..A-7 digest patches are enforced as per-unit stub_ban lines + REDTEAM blocks,
  not as a new lettered doctrine family (deltas are earned; existing doctrine covers this track). A-1 in
  particular BINDS: every P-N must pass corpus_composition_check (N-2) at authoring, ride N-4's one claim object,
  N-6's one lattice, N-11's fold substrate — a second provenance scale or a second scheduler is an authoring FAIL.

PROOF_SPINE: none (product-surface / distribution class) for ALL of P-1..P-8. Owner-acked 2026-07-07 per PSR-2.
  P feeds no PS-1..PS-6 number directly; it is a product surface. Per digest A-7 + K-10/GATE M-0 discipline: P is
  AUTHORED now, EXECUTED after the spine (N-1, PS-1..PS-6) — authoring now, executing after the spine, is the
  corpus-consistent order.

DONE DEFINITION (the FSM may not reach COMPLETE without): cargo check GREEN; auditor PASS on the affected crates
  only; wiring_contract_check.py --run-trigger PASS; the wire checker clean (or a WIRE-DARK[P-N] row);
  cross_lang_diff for P-2/P-5 frontend pairs; corpus_composition_check (N-2) PASS; the R10 integration smoke
  green; memory-writer ran.

================================================================================
THE 8 SESSIONS
================================================================================
  P-N   title                                    lever                        composes-with                 effort  wave
  P-1   app-fleet layer (health-tick + repair)   fleet-grain-over-taskspec    O-2,G-10,F-9,F-5,L-14,N-7,M-4  M-L     1   [KEYSTONE]
        (AppManifest + FleetRegistry + O-2 health-tick TaskSpecs + G-10 per-app canary + gated auto-repair)
  P-2   inter-tool ports + world-grain bus       typed-ports-over-canvas      P-1,O-4,N-6,G-1/G-2,M-5/M-6    M       2   depends P-1,O-4
        (PortSchema on the manifest + provenance/taint-carrying tool->tool edges + wire-dark-made-visual world grain)
  P-3   persona/mechanism split + friend-export  mechanism-vs-persona         P-1,P-2,M-5/M-6,L-12,G-9,FS-1  M       3   depends P-1,P-2
        (factor app into mechanism vs persona-data + `elai app export/import` re-verified on the recipient seat)
  P-4   living external brains                    mount-LIVE-not-static        O-3,B-35,F-5,N-6,D-5,P-6       M-L     2   GATED (Obsidian RECON)
        (LivingStoreContract per store class + change feeds + F-5/N-6-tagged ingest + O-3-claim-gated writes)
  P-5   operator surface + graduated ceremony     graduated-ceremony-profile   K-5,K-4,N-7,P-1,GD-4,W-2       M       1
        (toy/personal CeremonyProfile: gates observe-only, one-line receipts, nothing skipped + non-coder render)
  P-6   life-data privacy lattice                 out-flow-privacy-dimension   N-6,N-2,G-1,G-9,E-20,O-3,H-49  M       2   depends N-6,O-3
        (which TOOL may read which NAMESPACE: a domain dimension ON N-6 + per-tool read grants, default-deny)
  P-7   per-user operating profile                measured-asymmetric-process  N-3,H-17/H-18,M-2,GD-4,E-5     M-L     3   depends N-3 landed
        (adapt autonomy/strictness/verbosity per domain; loosen ONLY on measured own-run FP evidence; 3 defaults)
  P-8   small-team collaboration grain            shared-world-2-to-10-seats   O-3,K-23,I-2,M-2/M-3,3H        L       4   depends O-3,K-23
        (the hard kernel: a defined lossless deterministic two-seat SSB-stream merge + per-seat/agent provenance)
  (P-9..P-11 = the command-center wave, allocated in the SECOND-WAVE prose above: connector-pack format / ingress
   adapters + credential vault / workflow-from-goal. No table row here; the prose block is the allocator for them.)

================================================================================
THE COMPUTER-USE-STEAL SESSIONS (P-12..P-16 + P-23/P-24, Track COMPUTER-USE-STEAL)
================================================================================
  P-N   title                                    lever                        composes-with                 nature      effort  wave  PROOF_SPINE
  P-12  desktop/GUI operator lane                 gui-actuation-under-gates    E-14,J-1/J-8,G-1,G-2,4F,H-13   NEW         L       CU-2  none(product)
        (cua sandbox vendored substrate + OmniParser Tier-3 pixel fallback + frameId/display fail-closed binding +
         per-action screen-recording evidence + crosshair/zoom crops, all under ELAI gates; cua's own gating is a TODO)
  P-13  perception provenance + set-of-marks id   id-indirection-over-coords   E-14,J-1,J-8,J-77,G-1          EXTENSION   M       CU-1  none
        (VLM picks an integer off a source-tagged element list, never coords; axtree_exact|ocr|vlm_caption|pixel tag;
         Tier-3 pixel fires only on empty/incomplete AXTree; counts never from a VLM — J-1 doctrine unchanged)
  P-14  explore-then-crystallize record->replay   agentic-run-mints-artifact   E-14,O-2,G-2,C-crystallizer   EXTENSION   M-L     CU-2  FEEDS PS-5
        (an exploration run emits a deterministic Playwright/pyautogui script AS an O-2 TaskSpec stage + G-2 RRA
         binding script->minting-run; replay = execution mode. prior art cua replay_env, agent-zero scripts/)  depends P-13
  P-15  verified case-graph hardening             gate-the-save-not-every-run  J-96,K-18,K-19,I(evidence-search) EXTENSION M       CU-3  FEEDS PS-5
        (port agent-s S2 KnowledgeBase retrieve/distill/fuse but gate the SAVE behind J-96 mint-on-proof; add a bBoN
         narrative judge OVER JOURNAL ROWS as an evidence-ranked tie-breaker. cross-links J-96, J CLOSED)
  P-16  gated install catalog                     capability-on-demand-gated   P-9,P-10,L-12,N-6,P-12         EXTENSION   M       CU-3  none(distribution)
        (named {tool,source,mirror,alias,license_tag} rows + installPolicy behind the L-12/skillspector admission
         family; "download blender on demand" = a P-9 headless code-pack, P-12 GUI stage is last resort)  depends P-9,P-10
  P-23  journaled co-browse lane                  co-browse-with-receipts      H-13/H-15,E-14,P-13,P-14,P-6,P-10 EXTENSION M       CU-4  none(product)
        (canvas browse node over the E-14 engine: Tier-1 read extraction + Tier-2 CDP-screencast co-browse; ONE
         InputProxy journals every actor-tagged input before dispatch — no unjournaled input path; closes eval
         ENTRY 14, the Claude Code in-app-browser gap)  depends P-13
  P-24  mobile design/test lane                   mobile-as-a-dimension        H-14,E-14,P-12/13/14/16,P-23   EXTENSION   L       CU-5  none(product)
        (the deferred H-14 RN/Expo slice as 3 rungs: DeviceProfile matrix on E-14/H-14/P-23; Expo-web click-to-edit
         through the H-14 proxy; native emulator as a P-12 backend + mobile_axtree P-13 tag + P-14 Maestro-flow
         crystallize (actuation-only, evidence-leg verdicts); typed render_source per row, no rung laundering)  depends H-14,P-12,P-13,P-14,P-16

================================================================================
THE OPENCLAW-STEAL SESSIONS (P-17..P-22, Track OPENCLAW-STEAL)
================================================================================
  P-N   title                                    lever                        composes-with                 nature      effort  wave  PROOF_SPINE
  P-17  provider failover + honest economics      failover-semantics-package   J-28,I-50,Q-4,B-37,K-2        EXTENSION   M       OC-1  FEEDS PS-4
        (failure-class cooldown taxonomy + session-stickiness + persist-before-retry + strict-explicit-selection +
         never-merge billed-truth-with-estimate as typed end-to-end fields)
  P-18  durable taskspec exec + drift-binding     durable-runs-anti-toctou     O-2,O-4,P-11,3A,4G,G          EXTENSION   M-L     OC-2  none(product)
        (revision-counter concurrency + sticky-cancel + retry-ladders + watchdog for the run store; approval
         drift-binding: every approval binds a hashed run-plan + operand fingerprints, drift = deny)  depends P-11
  P-19  config/ops lifecycle + elai doctor        recoverable-config-and-state fail-closed-cfg,J-cfg,M,3H     NEW-area    M-L     OC-2  none(ops)
        (fail-closed-default-object config + last-known-good/rejected-write/shrink-guard; doctor detect/repair/lint +
         stable exit codes; VACUUM-INTO verified SSB backups + restart continuation queue; compaction split-rule)
  P-20  context/memory lifecycle hardening        no-context-lost-uncached     H-50,H-51,H-mem,L,H-MCP        EXTENSION   M       OC-1  none
        (pre-compaction memory-flush as an ordered journaled step + versioned on-demand skill/rules index +
         Dreaming promotion-gate prior art + per-provider cache-control mapping table)
  P-21  evidence / gate-credibility hardening     prove-the-gate-still-bites   Q-6,G-9,M,wiring-R10           EXTENSION   M       OC-1  FEEDS PS-6
        (verify-the-verifier seeded canaries that hard-block on a missed canary + missing-artifact=FAIL manifest +
         skipBackfill cross-lane dep + STRUCTURAL redaction + posture labels on smoke evidence)
  P-22  control-plane / channel hardening          fenced-returns-signed-ingress 3C,G-1,J-ctrl,P-10,M,4C       EXTENSION   M-L     OC-3  none
        (child-output-as-fenced-evidence + depth-scoped tool-strip + 9-step channel-binding precedence + pairing
         codes + dmScope + challenge-signed pairing + unknown-event fail-closed gating + idempotency keys + hook tier)

================================================================================
CANONICAL CONSTANTS + SCHEMA_VERSION BUMP SITES
================================================================================
Each P-N that adds an SSB action_type bumps SCHEMA_VERSION at the LIVE baseline. NEVER hard-code the baseline —
  re-read ssb.rs + action_type_lock.rs at edit time (W-9). The executor quotes "baseline+N from LIVE X/Y as of
  <edit date> (file:line)" in the decision doc. New rows by unit:
  P-1 fleet_health_tick | P-2 tool_port_edge | P-3 app_export / app_import | P-4 vault_ingest / claim_violation |
  P-5 ceremony_profile_set | P-6 privacy_grant / privacy_read_refusal | P-7 operating_profile_change |
  P-8 seat_journal_merge + seat_id/agent_id COLUMNS on the SSB row (P-8 M2 is the schema-invasive one).
  P-12 operator_action / operator_frame_bind | P-13 EXTEND J-1 PerceptionScaffold with a source/id-index field
  (net 0..+1, prefer-reuse J-1/J-8 rows) | P-14 crystallized_stage_minted | P-15 EXTEND J-96 case rows with a
  reuse-hit field (net 0..+1, prefer-reuse J-96 recurrence rows) | P-16 install_catalog_admit | P-17
  provider_failover_event (+ typed billed/estimate fields on the J-28 UsageTurnEvent, EXTEND not new) |
  P-18 taskspec_run_revision / approval_drift_reject | P-19 config_write_rejected / doctor_finding |
  P-20 memory_flush_event | P-21 EXTEND Q-6 corpus verdict with a posture-label + verifier-canary field (net 0..+1) |
  P-22 subagent_return_fenced / idempotency_replayed | P-23 cobrowse_session / cobrowse_input | P-24 emulator_session (+ device_profile / render_source fields EXTENDING axtree_snapshot / operator_action / cobrowse_session rows, prefer-reuse net +1). Each unit re-reads the LIVE SCHEMA_VERSION at edit (W-9);
  the prefer-reuse/EXTEND units bump 0 and say so.

================================================================================
SEQUENCING (advisory; digest section 6 waves + PSR-1 order binds)
================================================================================
WAVE 1 (composition, cheap, parallel-safe): P-5 (ceremony profile — K-5/K-4 config + N-7 render) + P-1 (fleet
  registry/health-ticks — O-2 TaskSpecs + manifests). P-1 is the KEYSTONE (every other unit binds to its
  manifest). They touch different seams (orchestrator gate-mode vs registry) so they are parallel-safe.
WAVE 2: P-2 (ports + world-bus edges — depends P-1 manifest + O-4 canvas) + P-6 (privacy lattice — depends N-6,
  O-3; the enforcement wall P-2/P-4 both need) + P-4 (living brains — AFTER the Obsidian RECON).
WAVE 3: P-3 (export bundle + persona split — depends P-1/P-2) + P-7 (operating profile — depends N-3 landed).
WAVE 4: P-8 (the shared-journal merge design spike, then the team grain — the one hard kernel).
Every wave carries its A-2 falsifier from day one (fleet-health green %, wired-edge count, exported-tool
  re-verification success rate, toy-tier measured gate tax, profile-promotion FP rate, merge determinism).

================================================================================
HARD RULES INHERITED FROM CLAUDE.md + F-0/G-0
================================================================================
blake3 not blake2/sha2; no .unwrap()/.expect() on prod paths; scoped tests (-p affected crates only, auditor runs
  them); anti-slopsquatting on any new dep (none expected; P-4 Obsidian/Notion surfaces are RECON-gated, never
  training-asserted); closed enums deny_unknown; kill-switch not enable-flag (W-2) for any new gate; every
  external claim WebSearch-verified at authoring (P-4 store APIs + P-8 merge/CRDT approach are the RECON items).

================================================================================
CARRIED DESIGN-DIGEST CORRECTIONS (authors use these, not memory)
================================================================================
- P-1 gated auto-repair is a CHANGESET (passes the same wiring/claim gates), never a silent patch (P-1 ABORT).
- P-2/P-6 ride N-6's ONE lattice; a second provenance/privacy scale is an authoring FAIL (A-1 / N-2).
- P-4 authors NO code against a training-asserted store API; the Obsidian/Notion RECON is BLOCKING (P-4 ABORT).
- P-5 observe-only still RUNS + JOURNALS the gate; the toy tier reduces ceremony, never skips evidence (W-2/R1).
- P-7 loosens ONLY on measured own-run FP evidence; user affect never loosens a gate (A-4 / P-7 ABORT).
- P-8 merge is lossless + deterministic + local-first; NO last-writer-wins, NO org SaaS control plane (P-8 ABORT).
- P-12 cua's own in-loop safety gating is a stubbed TODO (comparison ENTRY 10): the division of labor is cua brings
  the VM, ELAI brings the fail-closed gate; the operator-action stage is a taint source (G-1) + an operator lane
  (never load-bearing infrastructure a workflow's correctness rests on). OmniParser icon_detect weights are AGPL
  (Florence caption weights MIT) — a productization license anchor to re-verify at edit, NOT a training assertion.
- P-13 counts NEVER come from a VLM (J-1 law); the pixel/OCR/caption tags are ADVISORY provenance, axtree_exact is
  the only authoritative source; a per-crop caption never stands in for element identity (OmniParser S8 anti-pattern).
- P-14 openclaw's jsonl-replay is INPUT-through-a-mock, NOT a real record->re-score (comparison C4); cite cua
  replay_env as the record->re-score prior art, not openclaw. The crystallized script is the artifact; the RRA binds
  it to the run that minted it (a script with no minting-run receipt is refused).
- P-15 agent-s saves EVERY run's summary with NO success gate (its exact gap); ELAI GATES the save behind J-96
  mint-on-proof. Do NOT copy agent-s's ungated KnowledgeBase commit; the bBoN judge runs over JOURNAL ROWS, never
  over screenshots (we have ground-truth deltas they lack).
- P-16 the sanctioned "download blender/photoshop" form is a P-9 HEADLESS code-pack (blender --background --python),
  computer-use GUI driving (P-12) is the LAST resort, never the mechanism (TEIKOKU-asset-studio convergence).
- P-18 the lobster resume-token "approve+resume never re-runs prior steps" claim is UNVERIFIED-external (deep source
  read C1: the engine is an un-vendored npm type-stub; the in-repo wrapper force-errors needs_input). The VERIFIED
  mechanism is the Task-Flow revision-counter (S6/ENTRY13 amendment). Author toward a REAL subprocess-checkpoint
  model (SIGKILL + pipe backpressure + a step-checkpoint store), stronger than their cooperative enforcement; carry
  the resume-token as a RECON_NEEDED idea to re-derive, never "openclaw shipped this".
- P-19 openclaw copies full process.env into the workflow context with NO allowlist (C2); do the OPPOSITE (env
  allowlist). Adopt their cwd path-jail (case-fold-before-compare on win32) verbatim — it is correct.
- P-21 openclaw's qa "green report" default is fail-closed on UNKNOWNS ONLY, not on classified failures (C3): copy
  the unknown=FAIL posture, do NOT copy the default that lets a classified product-bug pass. The audit ledger is
  structurally metadata-only (V2) but the trajectory bundle in the same repo is CONTENT-carrying (V3) — keep the
  shareable export projection and the full debug bundle SEPARATE BY TYPE so they cannot be conflated.
- P-22 openclaw ships ClawHub with NO default install-time code gate + injection scoped OUT (V5) — this is the
  positioning EXHIBIT, not a thing to port; do NOT copy their advisory regex scanner (signature-match with no teeth
  is the pattern our gate doctrine rejects). ELAI's install path (P-16) is fail-closed via the L-12 admission family.

FIFTH WAVE — CREATIVITY-FALSIFIER (P-25, authored 2026-07-14): a distinct source from every prior P wave —
  not a product-surface or steal unit but the user-directed successor falsifier to the Q-10 B-CREATIVITY-0
  REFUTED closeout (workflow/benchmarks/research-creativity-b0v3-2026-07-14/). Source: the Q-10 rejection data
  + the pilot P-A 3.5x real-anchor receipt + novelty-engine D1(a) (the one measured surviving creativity lever =
  exogenous grounded information). J is CLOSED, P is the open letter, so the successor lands here cross-linked
  to the Q-10 track.
  - P-25 exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0): SIX arms in TWO
    independently-preregistered gate families on one frozen corpus of K=30 open-world real ELAI improvement
    fixtures with hidden deterministic micro-executors. FAMILY A (injection): deterministic MODEL-FREE
    target-relevant artifact retrieval (elai.map slice + measured row + VerifiedCitation) injected into the
    single direct champion call, vs DIRECT / cross-subsystem EXO_SHUFFLED / PAD_MATCHED, HARD 1.3x cost cap
    (the 8.8x killer preregistered away). FAMILY B (grounding-as-exit loop-back, per
    workflow/reasoning/grounding-as-exit-grc-v3-2026-07-14.md): TRIAGE_REVISE (generate -> deterministic
    public triage kill-reasons -> ONE revision) vs SHUFFLED_REVISE (permuted kill reasons, same template),
    HARD 2.5x cap + cost-normalized non-inferiority. Co-primaries per family = execution-validated mechanism
    yield + best target value. Harness-only; PROOF_SPINE: none (user-acked in-session). A SUPPORTED family
    authors its own separate adoption unit; BOTH families NULL/REFUTED closes the creativity line entirely.
    Red-teamed TWICE by adversarial-auditor (authoring pass + six-arm delta pass; all BLOCKs patched in file,
    incl. the card-grammar/triage-leg reconciliation catch); plan-gate PASS. UNLIKE P-1..P-24 this unit is
    NOT spine-gated: it is itself a FREE-lane evidence run (~280 calls), executable on user go.

================================================================================
SESSION LOG (populated as P-Ns ship)
================================================================================
(empty at generation)

================================================================================
NEXT STEP
================================================================================
Execution is GATED behind the proof spine (A-7): author now, execute after N-1 + PS-1..PS-6. When cleared,
  WAVE 1: P-1 (KEYSTONE) + P-5 in parallel (different seams). For P-1, run the RECON greps (G-10 canary
  parameterization + the O-2 TaskSpec factory API + F-9 dep-checker entrypoint) BEFORE the first edit.

End of P-0_INDEX.
