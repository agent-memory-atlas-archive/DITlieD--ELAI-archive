# O-Plan Index — the PERSONAL RESEARCH MACHINE side, wiring-enforced (4 sessions; Track RESEARCH-MACHINE authored)
Date: 2026-07-06 (authored from the spec-eval design digest; owner-acked PROOF_SPINE: none)

Source synthesis: workflow/reasoning/elai-personal-research-machine-2026-07-06.md (the AS-SPECIFIED spec-eval
  of "give ELAI a TEIKOKU-style auto-research mode + brain + scheduled tasks composed on the canvas = a personal
  task n8n-with-receipts"). The digest's four layers L1..L4 are 1:1 with O-1..O-4 below. Its adversarial pass
  A1..A5 and its verbatim-steal list feed each unit's REDTEAM + stub_ban.
Cross-source: workflow/handoff/teikoku-research-machine.md (the LIVE-proven enum-gated classify + injection
  canaries + confidence-per-sink + write_page/upsert_page namespace patterns O-1/O-3 steal verbatim).

Predecessors: B-plan .. N-plan. O COMPOSES-WITH them (GD-1); restates none. O is a PRODUCT-SURFACE letter: unlike
  N (interlace/unify existing units, invent nothing), O ships a NEW user-facing feature (research daemon + typed
  scheduler + mounted brain vault + canvas workflow mode) that is ~80% composition of shipped subsystems (B-42
  Cosmos, C-9 task_runner, 3F rules-engine, E-5 memory, D-5 privacy, G-1 taint, F-3 tool-filter, G-2 VCR,
  H-12/J-31/J-60 canvas) plus a small set of genuinely new mechanisms (a closed TaskSpec vocabulary, a vault
  namespace-claims contract, the WORKFLOW canvas mode, the Cosmos manifest generalization).

Predecessor reading (mandatory before any O-N execution):
  workflow/reasoning/elai-personal-research-machine-2026-07-06.md (the WHY: the four layers + the A1..A5 attacks).
  docs/thefinalglance/ELAI-FEATURE-MAP.md (the composition edges each O-N consumes — the single context load;
    do NOT re-read the legacy 12-file list).
  docs/thefinalglance/F-0_INDEX.md (W-1..W-10 wiring doctrine, INHERITED verbatim).
  docs/thefinalglance/G-0_INDEX.md (GD-1..GD-5; O is a compose-not-restate letter like G/N).
  this file (O-0_INDEX.md), then O-SUMMARY_INDEX.md, then the target O-N detail file.

## Why O-plan exists (read this before anything else)
TRACK RESEARCH-MACHINE (O-1..O-4). Authored 2026-07-06 from the spec-eval digest. The CENTRAL FINDING (AS-
  SPECIFIED, grounded against the feature map): the "personal research machine" the owner wants is architecturally
  CHEAP for ELAI because it is mostly COMPOSITION of already-shipped subsystems, and it lands on ELAI's actual
  differentiator (journaled, gated, replayable runs). The one WRONG way to build it is as a generic dataflow
  engine (n8n-minus-integrations); the RIGHT shape is a CLOSED-vocabulary typed task layer where every stage kind
  is code and every fire is an SSB row. Each unit is BUILD-on-shipped, not net-new:
  - O-1 generalizes the Cosmos manifest (already a scheduled research loop) to a user-editable typed topic set +
    threads the G-1/F-3/G-2 trust spine through every fetched byte. ADDS the typed-topic + enum-classify surface.
  - O-2 adds the typed TaskSpec scheduler (rules-as-data in SQLite, cron/ssb/manual triggers, journaled fires,
    intervention-card inbox), reusing the task_runner interval + SSB temporal-monitor. ADDS the schedule layer.
  - O-3 mounts a portable markdown brain vault (not converts) with a NEW namespace-claims contract so ELAI +
    teikoku share one vault; memory_facts stays the machine grain. ADDS the user-grain vault surface.
  - O-4 adds a WORKFLOW canvas mode: editor + monitor over TaskSpecs, wire-dark made visual, dispatch through the
    FSM (J-31), never a canvas runtime. ADDS the composition surface.

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
INHERITS GD-1..GD-5 (canon: G-0_INDEX.md): COMPOSE-never-restate (every O-N names what it consumes and must NOT
  re-implement); GD-4 observe-only-first for any gate defaulting ON; GD-5 honest residue named in-file.
ADDS no new delta family. The A1..A5 digest patches are enforced as per-unit stub_ban lines + REDTEAM blocks,
  not as a new lettered doctrine family (R-j: deltas are earned; existing doctrine covers this track).

PROOF_SPINE: none (product-surface / distribution class) for ALL of O-1..O-4. Owner-acked 2026-07-06 per PSR-2.
  Nearest spine number is PS-3 (hallucination delta) only if research briefings counted as grounding inputs —
  the digest rates that WEAK and does not claim it. O feeds no PS-1..PS-6 number directly; it is a product surface.

DONE DEFINITION (the FSM may not reach COMPLETE without): cargo check GREEN; auditor PASS on the affected crates
  only; wiring_contract_check.py --run-trigger PASS; the wire checker clean (or a WIRE-DARK[O-N] row);
  cross_lang_diff for O-4's frontend pair; the R10 integration smoke green; memory-writer ran.

================================================================================
THE 4 SESSIONS
================================================================================
  O-N   title                                    lever                      composes-with                 effort  wave
  O-1   research daemon (generalize Cosmos)       manifest-generalization    B-42/C-16,G-1,F-3,G-2,B-40    M-L     1   [KEYSTONE]
        (typed topic set + enum-gated classify + taint/tool-filter/VCR threading; ships steal-scan + monitor kinds)
  O-2   scheduled TaskSpec substrate              rules-as-data-scheduling   3F,C-9,SSB/K-14,inbox         M-L     1
        (typed TaskSpec in SQLite, cron/manual triggers, journaled fires, dashboard intervention-card inbox)
  O-3   brain vault contract + mounting           mount-not-convert          E-5,D-5,teikoku brain_store   M       2
        (markdown vault contract + namespace-claims hard-refusal + guarded upsert; memory_facts stays machine grain)
  O-4   canvas WORKFLOW mode                      canvas-as-spec-editor      H-12,J-31,J-60,O-2,O-1        M       3
        (editor+monitor over TaskSpecs, enum-constrained palette, wire-dark visible, dispatch through the FSM)

================================================================================
CANONICAL CONSTANTS + SCHEMA_VERSION BUMP SITES
================================================================================
Each O-N that adds an SSB action_type (O-1 classify row, O-2 task_spec_fired, O-3 vault_write/claim_violation,
  O-4 reuses O-2's fire row) bumps SCHEMA_VERSION at the LIVE baseline. NEVER hard-code the baseline — re-read
  ssb.rs + action_type_lock.rs at edit time (W-9). The executor quotes "baseline+N from LIVE X/Y as of <edit
  date> (file:line)" in the decision doc.

================================================================================
SEQUENCING
================================================================================
WAVE 1 (substrate, parallel-safe): O-1 (daemon) + O-2 (scheduler). O-1 is the KEYSTONE — the trust-spine
  threading is the ELAI-over-teikoku delta and everything else sinks into it. They touch different crates
  (knowledge vs orchestrator/rules) so they are parallel-safe.
WAVE 2: O-3 (vault) — independent; can land any time after WAVE 1 gives it a FileToBrain caller (or ship with an
  interim `elai brain write` CLI caller).
WAVE 3: O-4 (canvas) — depends on O-2's TaskSpec store being queryable (else ships against a read-only fixture
  projection per its FORK).

================================================================================
HARD RULES INHERITED FROM CLAUDE.md + F-0/G-0
================================================================================
blake3 not blake2/sha2; no .unwrap()/.expect() on prod paths; scoped tests (-p affected crates only, auditor runs
  them); anti-slopsquatting on any new dep (none expected); closed enums deny_unknown; kill-switch not enable-flag
  (W-2) for any new gate; every arXiv/external claim WebSearch-verified at authoring (this track cites no arXiv).

================================================================================
CARRIED DESIGN-DIGEST CORRECTIONS (authors use these, not memory)
================================================================================
- the manifest generalization is a SWAP of the topic source, NOT a rewrite of CosmosStage (O-1 FORK covers the
  hard-inlined case).
- the wall-clock trigger stays SUPERVISOR-PLANE, outside the replay set (K-14) — O-2 ABORT enforces.
- the vault is MOUNTED, never converted into memory_facts (O-3 ABORT + stub_ban).
- the canvas has NO runtime — dispatch is the FSM (J-31) — O-4 ABORT + stub_ban.

================================================================================
SESSION LOG (populated as O-Ns ship)
================================================================================
(empty at generation)

================================================================================
NEXT STEP
================================================================================
WAVE 1, O-1: run the RECON_NEEDED greps (cosmos_worker manifest type + G-1 taint wrap API), then generalize the
  manifest per the O-1 MOVES. Dispatch O-1 + O-2 in parallel (different crates).

End of O-0_INDEX.
