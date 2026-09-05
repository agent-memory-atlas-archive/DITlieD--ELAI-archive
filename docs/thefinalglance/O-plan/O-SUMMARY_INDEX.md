# O-plan — Summary Index of all 4 sessions (Track RESEARCH-MACHINE)
Generated: 2026-07-06. Mirrors N-SUMMARY_INDEX format.

Each entry preserves: the lever + source finding (digest L1..L4), grounding strength, the shipped subsystem it
EXTENDS (with the plan-id), the genuinely-new mechanism it ADDS, the COMPOSES-WITH artifacts consumed, the
production entrypoint, and the observable_effect done-gate. Doctrine + sequencing live in O-0_INDEX.md; the
source design is workflow/reasoning/elai-personal-research-machine-2026-07-06.md.

## Premise
The owner's "TEIKOKU-style personal research machine on the ELAI canvas" decomposes into four vertical slices
that are ~80% composition of shipped subsystems. The differentiator vs teikoku (which has the classify canaries)
and vs n8n (which has ungated arbitrary nodes) is the same in every unit: gated + journaled + replayable +
CLOSED-vocabulary. O is a product-surface letter, PROOF_SPINE: none (owner-acked); it feeds no PS-1..PS-6 number.

==== TRACK RESEARCH-MACHINE ====

### O-1 — research daemon (generalize Cosmos) [KEYSTONE]
Track RESEARCH-MACHINE | lever manifest-generalization | grounding MODERATE (index-derived, PLAN-asserted) |
  nature BUILD-on-shipped | effort M-L | WAVE 1
Steal/build: CosmosWorker is ALREADY a scheduled research loop (manifest, hourly tick, FlashResearch refresh,
  time-decay, <=1500-token briefing, CosmosStage) with the manifest hard-bound to designer briefs. O-1 generalizes
  ONLY the manifest to a user-editable typed topic set (closed `kind` enum: steal-scan|leaderboard|discovery|
  monitor) and threads G-1 taint + F-3 tool-filter + G-2 VCR through every fetched byte. The per-topic pipeline
  (fetch->enum-classify->dup-precheck->file->digest) is teikoku-proven; ELAI adds taint + replay on top of the
  canaries. What is dark/absent: the typed-topic surface + the enum-classify stage as ELAI code (teikoku-only today).
COMPOSES-WITH: cosmos_worker/CosmosStage/briefing-compiler (EXTEND, do NOT re-author), G-1/F-3/G-2, FlashResearch.
Integration: crates/knowledge cosmos manifest loader + classify stage; CosmosDrawer extended (no new drawer).
observable_effect: the live cosmos tick drives a steal-scan topic over a canary-injection fixture; a classify SSB
  row lands off-agent {class,confidence,taint_tier}; the canary routes to unsorted; NEGATIVE: an out-of-enum
  classifier output or a canary reaching a steal sink FAILS.

### O-2 — scheduled TaskSpec substrate
Track RESEARCH-MACHINE | lever rules-as-data-scheduling | grounding MODERATE (index-derived, PLAN-asserted) |
  nature BUILD-on-shipped | effort M-L | WAVE 1
Steal/build: Codex Automations (time/event schedules -> inbox -> auto-archive) but TYPED + JOURNALED. `TaskSpec`
  is a SQLite row like the rules engine: trigger (Cron|SsbEvent|Manual), pipeline (ordered CLOSED enum of stage
  kinds), sink (VaultPage|Digest|DashboardInbox|Toast). Every fire is a journaled SSB row carrying the trigger
  cause; K-14 keeps the wall-clock trigger supervisor-plane so the run stays deterministic; missed-run detection
  reuses the SSB temporal-monitor; the inbox is the shipped intervention-card surface. What is absent: the typed
  spec + the trigger-to-task_runner wiring.
COMPOSES-WITH: rules-engine rows-as-data (3F), task_runner interval (C-9), SSB/K-14 temporal-monitor,
  intervention-cards, O-1 stage kinds.
Integration: crates/orchestrator (or rules crate) TaskSpec table + dispatcher + fire-row emit.
observable_effect: a Cron TaskSpec due-now fires exactly once per tick, emits task_spec_fired {spec_id,
  trigger_cause, stage_path}, renders an inbox card for a result-bearing fire; NEGATIVE: an empty fire that still
  cards, or a double-fire, FAILS.

### O-3 — brain vault contract + mounting
Track RESEARCH-MACHINE | lever mount-not-convert | grounding MODERATE (index-derived, PLAN-asserted) |
  nature BUILD-on-shipped | effort M | WAVE 2
Steal/build: split the brain by grain — E-5 memory_facts stays the machine-grain prompt store; a deterministic
  markdown vault (topics/<t>/<slug>.md + frontmatter + wikilinks + living rankings/queue/index) is the user-grain
  surface, stealing teikoku brain_store write_page/upsert_page discipline verbatim. Brain migration costs ZERO
  conversion because the CONTRACT is the format; ELAI mounts the existing teikoku vault. NEW MECHANISM: a
  namespace-claims file at the vault root -> a write outside a claimed namespace is a hard refusal (so two
  harnesses share one vault). Privacy: a D-5 life-layer denylist keeps personal topics out of project/cloud prompts.
COMPOSES-WITH: E-5 memory_facts (disjoint machine grain, do NOT dual-write), D-5 denylist + privacy layer,
  teikoku brain_store contract, O-1/O-2 FileToBrain sink.
Integration: crates/knowledge vault reader/writer + namespace-claims parser + guarded upsert.
observable_effect: a FileToBrain write into an ELAI-claimed namespace creates a valid page (vault_write row); an
  attempted write into an unclaimed namespace lands a claim_violation row; NEGATIVE: any unclaimed-namespace write
  or any mutation of a user note FAILS.

### O-4 — canvas WORKFLOW mode
Track RESEARCH-MACHINE | lever canvas-as-spec-editor | grounding MODERATE (index-derived, PLAN-asserted) |
  nature BUILD-on-shipped | effort M | WAVE 3 | depends O-2
Steal/build: a WORKFLOW mode on the shipped canvas mode bar (H-12), same shape as J-31's doctrine (nodes = bound
  units, edges = verified claims, dispatch through the NORMAL FSM, NO canvas runtime) over O-2's TaskSpecs. Node =
  a typed stage instance; edge = dataflow carrying a provenance tier; node chrome = last-run SSB state (J-60);
  editing a node edits the spec row CONSTRAINED to the closed enum. A node with a missing consumer renders
  VISIBLY DISCONNECTED — wire-dark made visual. "n8n with receipts". What is absent: the WORKFLOW mode + the
  TaskSpec->nodes projection + the enum-constrained palette.
COMPOSES-WITH: H-12 mode bar (EXTEND), J-31 FSM-dispatch doctrine, J-60 overlay, O-2 store + enums, O-1 tiers.
Integration: crates/dashboard-v2 Svelte WORKFLOW view + Rust list_task_specs/get_task_spec_runstate Tauri commands.
observable_effect: selecting WORKFLOW in the live dashboard renders a TaskSpec as nodes+edges with J-60 chrome, a
  missing-consumer node renders disconnected, run-now emits O-2's fire-row via the FSM; NEGATIVE: a canvas-side
  execution path or a normally-connected wire-dark node FAILS.

## Track cross-cutting (what makes it different)
Every unit enforces the SAME three digest patches as stub_ban + REDTEAM: A1 (no free-text classifier output to a
prompt — closed enum + confidence + frozen canaries), A3 (no generic/custom node — closed stage vocabulary, new
kinds ship as code), A4 (two-masters — ELAI ships the SUBSTRATE, topic packs are data, disjoint vault namespaces).
A2 (default-off graveyard) is the whole-feature falsifier: the digest routes success to a measured digest-open /
For-You-click engagement signal; zero engagement after N days decays the topic honestly.
