# J-60: process-layer observability canvas — agents and FSM as bound nodes, tabs are zoom, all derived from the journal

STATUS: PLANNED (WAVE CFW-3; tier L; BUILD-on-shipped — observability/read only, redirect deferred to J-61)

- Track J | lever OBSERVABILITY+CANT-WRECK | grounding STRONG (FSM/agent_pool/canvas substrate code-verified this session via elai.map; worktree leg PLAN-asserted on PLANNED I-2) | nature BUILD-on-shipped | effort L | WAVE CFW-3 | depends: shipped H-13/H-15 canvas + J-31 (system-layer sibling); composes Tier-C (shipped) + I-2 (PLANNED worktree); phase_transition SSB (B-2 WIRED)

## Premise

J-31 made the SYSTEM layer (the user's codebase as bound, wiring-verified nodes). J-60 makes the
PROCESS layer: ELAI's OWN orchestration — FSM states, agent_pool workers, phase transitions —
projected as nodes on the SAME canvas at a different altitude. Nobody renders the harness's own
execution as a navigable graph the user watches live (n8n/Flowise render the user's flow, never the
builder's own agents). The user pain this serves, in the user's words: "when elai spawns a sub-agent
i want to see all it is working on, with its own codebase version, so i can drill into each agent's
work." J-60 makes each running agent a process node; descending it (semantic zoom) opens its scoped
slice + its worktree — that descend IS the per-agent tab. Everything is read-only and re-derived
from the journal (GD-2). This is the v1 observability falsifier for the layered canvas: prove people
WATCH ELAI work spatially before betting they STEER it (the redirect verbs are J-61). Root decision
locked in grill (workflow/reasoning/J-60-decisions.md): observability-first.

## Steal/build (6 legs)

1. PROCESS LAYER = a third altitude over J-31's node model. The canvas gains an altitude axis:
   PROCESS (FsmState / AgentRole worker / phase node) -> SYSTEM (J-31 bound units) -> SCRIPT
   (capability / tool / MCP leaf). One graph, semantic zoom, NOT three graphs. A process node
   operates-on the SYSTEM nodes its run touches; descending is the navigation primitive.
2. PROCESS NODE = JOURNAL PROJECTION, NOT STATE (the same anti-drift keystone as J-31 leg 2).
   ProcessNode { node_id, binding: FsmSession(session id) | AgentWorker(agent_id) | PhaseStep(phase
   id), display } derived from phase_transition rows (B-2) + agent_pool lifecycle/SSB rows. No
   parallel state file; the canvas re-derives byte-identically under ELAI_SEED (GD-2). The LLM never
   sets a node's status (TD-3) — journal rows are the only source.
3. TAB = DESCEND (the unification; grill decision). Opening an agent's tab IS zooming into its
   AgentWorker process node. A tab is a filtered VIEW of the one journal scoped to agent_id, never a
   first-class object holding its own state (forced by GD-2). The descend reveals: the agent's task
   + FSM substate, its tool-call stream (SSB rows), the SYSTEM/SCRIPT subgraph it touches, and its
   worktree diff (leg 4).
4. PER-AGENT WORKTREE VIEW. Each agent's tab renders its isolated codebase/worktree (the I-2 COW
   worktree, warm git-worktree pool on Win/macOS). The worktree is real working filesystem, merged
   at I-2's deterministic barrier — the canvas does not HOLD it, it PROJECTS it, so "no parallel
   state" holds (worktree = ground-truth working state, journal = the record, canvas = the
   projection). Until I-2 ships, the per-agent diff is derived from the journaled taint-tagged
   op-stream (the H-49 pre-image already captured at file_ops.rs:272/300 then discarded) — you see
   what the agent changed without a separate worktree. When I-2 lands the tab renders the live
   worktree and the diff SOURCE swaps; the UI does not. (Spec-eval as-specified: the full worktree
   view is the design; I-2 is a recorded build DEPENDENCY, never a reason to shrink it.)
5. EMIT-ONLY (no redirect in this unit). The engine projects, the user navigates — zoom, open tab,
   H-15 deixis-select with its G-1 taint-gated read-back — but issues NO graph mutation. The
   single-writer journal is read-only to the canvas (SsbViewer WAL poll). Redirect verbs (connect /
   cut / reorder / rebind / agent-control) and the user-preempt conflict model are J-61.
6. DEIXIS-ONLY DESCEND (the anti-node-soup discipline). A process node collapses to ONE labeled node
   until the user descends it; ascending re-collapses. Never auto-expand. This is what stops the
   "mess of nodes and edges" that kills Langflow/Flowise at scale (web-verified 2026-06-20: the
   documented complexity plateau of visual builders) — the user never sees N altitudes at once.

## Existing ground truth (verified this session via elai.map + reused from J-31 2026-06-12; re-grep at edit, W-7)

- FsmState @ crates/orchestrator/src/fsm.rs (transition + TransitionRecord @ fsm.rs:1572 +
  can_transition_to @ fsm.rs:1647; 9 states, B-2 phase_transition SSB WIRED — the action_type NAME
  is PLAN-asserted, executor re-greps ssb.rs).
- AgentPool @ crates/orchestrator/src/agent_pool.rs (AgentPoolConfig + max_concurrent @
  agent_pool.rs:4422; spawn -> agent_id UUID @ agent_pool.rs:4320; Erlang supervision); AgentRole @
  crates/orchestrator/src/ipc_gateway.rs:219 (8 roles incl Auditor/IdeaFramer/IdeaCritic).
- Tier-C process isolation SHIPPED (crates/agent + crates/ipc-types; agents = minimal OS processes,
  15-message postcard IPC) — the per-agent identity the tabs project.
- Canvas substrate (J-31-verified 2026-06-12): CanvasWindow.svelte; CanvasMode union @
  crates/dashboard-v2/src/lib/state/canvas.svelte.ts:39; InProcessCanvasDispatcher @
  crates/orchestrator/src/canvas_dispatcher.rs:101; CanvasOpApplied SSB live; spawn_view @
  production_agent_rpc.rs:1299-1332; CanvasViewEvent @ :3099; CanvasUiAction @
  canvas_ui_action.rs:61; G-1 taint SHIPPED; G-2 RRA @ rra.rs:476.
- H-49 pre-image capture @ crates/orchestrator/src/architecture/timetravel.rs:310 (capture_file, CAS-stored; EditHistoryEventPayload @ ssb/payloads.rs:19768) — the worktree-diff fallback source until I-2.
- I-2 COW worktree isolation PLANNED (zero worktree types in elai.map this session — confirmed
  absent); J-12 runtime intervention harness PLANNED (orthogonal; the agent-control verbs that need
  it are J-61).
- skill_system_event is DARK (E-43, ELAI_SKILL_SYSTEM_ENABLE) — skill-grain process nodes need the
  flag promoted; v1 uses phase_transition + agent-lifecycle grain ONLY, so it carries no E-43
  dependency.

## REVISED scope (W-10 tiered)

Tier-1 (ships first): process altitude + process nodes from phase_transition + agent-pool lifecycle
rows + tab=descend + worktree diff from the journaled op-stream + deixis-only descend. Read-only;
derivation runs auditor/idle-tick, NEVER on the render hot path; the canvas only renders rows.
Tier-2 (gated): live I-2 worktree render (when I-2 ships, swap the diff source); skill-grain process
nodes (when E-43 is promoted + allowlisted); cross-altitude operates-on edges auto-derived from run
provenance.

## Integration

- frontend: canvas.svelte.ts altitude axis + ProcessLayer/AgentTab renderers under
  src/lib/canvas/ (frontend_pair = the PROCESS-mode surface + per-agent tab; cross_lang_diff +
  svelte-check in-unit).
- dispatcher: READ-ONLY — no new mutation ops; reuse spawn_view / CanvasViewEvent for tab-open
  (production_agent_rpc.rs:1299).
- orchestrator: NEW crates/orchestrator/src/process_canvas.rs (~220 LOC): process-node registry
  deriving from phase_transition + agent_pool SSB rows + a worktree-diff projection over the H-49
  op-stream; verdict-free (observability only).
- SSB: PREFER-REUSE — no new mutation action_type. +ProcessViewProjected (Pattern-2 single-variant;
  Kind: ProcessNodeDerived / AgentTabOpened / WorktreeDiffRendered) ONLY if a telemetry beacon is
  wanted; else reuse CanvasViewEvent. baseline +0..+1 from LIVE 241/550 (ssb.rs:3152 /
  orchestrator/tests/action_type_lock.rs:705 per J-31 grep 2026-06-12; I-drain stamp ~268/577
  Tier-B — re-read at edit, W-9; full golden-lock 9-tuple lockstep if a variant is added).

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/process_canvas.rs — ProcessNode registry + journal derivation + worktree-diff projection
  - crates/dashboard-v2/src/lib/canvas/ProcessLayer + AgentTab renderers (the PROCESS-mode surface)
consumes_symbols:
  - crates/orchestrator/src/fsm.rs::FsmState/transition (phase_transition rows, B-2) — do NOT re-implement the FSM
  - crates/orchestrator/src/agent_pool.rs::AgentPool/AgentRole (worker lifecycle rows) — do NOT re-implement the pool
  - crates/orchestrator/src/canvas_dispatcher.rs::InProcessCanvasDispatcher + spawn_view (production_agent_rpc.rs:1299) — reuse for tab-open
  - crates/orchestrator/src/file_ops.rs:272/300 (H-49 pre-image) — worktree-diff source; G-1 taint for worktree taint badges
  - crates/orchestrator/src/system_canvas.rs (J-31 binding model) — the SYSTEM altitude this composes with; do NOT re-implement node binding
reachable_from:
  - crates/dashboard-v2 CanvasWindow PROCESS mode — a real user-facing window; tab-open routes through the spawn_view production path (production_agent_rpc.rs:1299) [PRODUCTION canvas]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO parallel canvas-side state (journal is the store; re-derive byte-identically, GD-2)
  - NO redirect / mutation verb in this unit (read-only; verbs are J-61)
  - NO LLM-set node status (derived from journal rows only, TD-3)
  - NO worktree diff fabricated from model output (source = H-49 op-stream or the I-2 worktree only)
  - NO auto-expand (deixis-only descend); NO .unwrap()/.expect() on prod paths; blake3 only
frontend_pair:
  the PROCESS-mode canvas surface + per-agent tab + the spawn_view/CanvasViewEvent Tauri read. cross_lang_diff + svelte-check in-unit.
observable_effect:
  trigger: "run any FSM task that spawns >=2 agent_pool workers; open the PROCESS canvas; descend an agent node"
  expect_ssb: "the phase_transition + agent-pool lifecycle rows the run already emits; (optional) ProcessViewProjected beacon on tab-open"
  expect_artifact: "each live agent renders as a process node; descending it opens its tab with the agent's scoped task/tool-stream + a worktree diff derived from the journaled op-stream; the full process graph re-derives byte-identically from the journal under ELAI_SEED (GD-2). NEGATIVE (R1/R2): a node whose status is set from model output (not journal rows) FAILS the contract test; an agent with zero journal rows renders NOTHING, never a fabricated node."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md --run-trigger
rg "struct ProcessNode" crates/orchestrator/src/process_canvas.rs            # process-layer derivation present
rg -c "set_status.*model|llm.*verdict|status.*from_model" crates/orchestrator/src/process_canvas.rs   # MUST be 0 (no LLM-set status)
rg -c "\.unwrap\(\)|\.expect\(" crates/orchestrator/src/process_canvas.rs    # prod paths MUST be 0 (test cfg excepted)
test -f workflow/reasoning/J-60-decisions.md
```

## ACCEPTANCE
- [ ] GD-2: the full process graph + every agent tab re-derive byte-identically from the journal under ELAI_SEED — no parallel state file exists.
- [ ] TD-3: every node status comes from a journal row or the deterministic worktree-diff; the LLM sets nothing.
- [ ] R1/R2: a zero-evidence agent renders nothing; a model-sourced status FAILS the contract; missing journal rows = FAIL not SKIP.
- [ ] W-2: ships ON (read-only observability has no kill-switch graveyard risk); no default-OFF ELAI_*_ENABLE introduced.
- [ ] Deixis-only descend: no auto-expand path exists; ascending re-collapses to one labeled node.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + dashboard-v2 svelte-check.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: observable_effect above — real PROCESS-mode window, off-agent journal rows, negative arm specified.
- W-2: N/A enable-flag (ships ON, read-only); justified — no gate to graveyard.
- W-6: dashboard item IS the deliverable (PROCESS-mode surface + tabs).
- W-7: every anchor re-grepped at edit; PLAN-asserted items (phase_transition action_type name, I-2 worktree) flagged.
- W-9: SCHEMA baseline re-read at edit (only relevant if the optional ProcessViewProjected variant is added).
- GD-1: composes J-31/H-13/H-15/Tier-C/H-49; re-implements none.
- GD-2: byte-identical re-derive under ELAI_SEED — checked.
- GD-5: honest residue — the live worktree render is gated on PLANNED I-2; until then the diff is op-stream-derived (named in leg 4).

## Tests to add
- crates/contracts/tests/j60_process_canvas_contract.rs (~120 LOC): (a) process nodes derive from a fixture phase_transition + agent-lifecycle journal; (b) re-derivation under ELAI_SEED is byte-identical; (c) a model-sourced status is rejected (TD-3 negative); (d) an agent with zero rows yields no node (R1); (e) the worktree diff comes from the op-stream/H-49 pre-image, not model text; (f) tab-open routes through spawn_view (reachability, not a test shim).

## Files touched
- CREATE: crates/orchestrator/src/process_canvas.rs (~220 LOC)
- MODIFY: crates/dashboard-v2/src/lib/state/canvas.svelte.ts (~15 LOC: altitude axis over the CanvasMode model)
- CREATE: crates/dashboard-v2/src/lib/canvas/ProcessLayer.svelte + AgentTab.svelte (~260 LOC)
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (~20 LOC: reuse spawn_view for tab-open; no new mutation op)
- CREATE: crates/contracts/tests/j60_process_canvas_contract.rs (~120 LOC)
- CREATE: workflow/reasoning/J-60-decisions.md (~40 LOC)

## Citations / Grounding
- ELAI ground truth (this session, elai.map): fsm.rs FsmState/TransitionRecord:1572/can_transition_to:1647; agent_pool.rs AgentPoolConfig/max_concurrent:4422/spawn:4320; ipc_gateway.rs:219 AgentRole. J-31 substrate anchors reused (code-verified 2026-06-12).
- Category grounding (WebSearch 2026-06-20): visual LLM builders (Langflow/Flowise/Rivel/n8n) plateau as "a mess of nodes and edges to manage in the UI" past a complexity threshold (zenml.io langflow-alternatives; langchain.com "Not Another Workflow Builder") — the deixis-only-descend + journal-derived discipline is the documented escape. AgentCoord (arXiv:2404.11943) and "From Static Templates to Dynamic Runtime Graphs" (arXiv:2603.22386) ground the process-layer-as-graph direction.
- CLAUDE.md hard rules: no .unwrap in prod (Rust); blake3 only; scoped tests (-p); main model never runs tests; vertical-slice wiring (producer+consumer+smoke).

## Open questions / risks
- The runtime agent class that actually edits code (Tier-C agents are spec'd "NO net/LLM/tools"): confirm at edit which AgentRole's worktree the tab renders — the executor role, not the auditor. If no runtime agent edits in an isolatable tree pre-I-2, the v1 worktree diff is per-FSM-session not per-agent; decide at executing.
- phase_transition SSB action_type name is PLAN-asserted (B-2 WIRED per feature map) — executor re-greps ssb.rs; if the grain is per-transition not per-phase the ProcessNode binding adjusts.
- Whether the optional ProcessViewProjected beacon earns its golden-lock cost vs reusing CanvasViewEvent — decide at authoring; default PREFER-REUSE (no new variant).

## Next
J-60 ships the watch half of the layered canvas: ELAI's own execution and every sub-agent's work
become a navigable, journal-true graph, with the per-agent worktree tab the user asked for. It
proves (or refutes) that people watch ELAI spatially. On that signal, J-61 adds the steer half — the
redirect verbs and the user-preempt conflict model — turning the projection into a bidirectional
negotiation surface.

End of J-60.

## WARGAME BLOCKS (refine-before-execute, injected 2026-07-25)

### EXPECTED_OBSERVATION
- After creating process_canvas.rs: cargo check -p elai-orchestrator compiles clean, ProcessNode struct present, zero .unwrap()/.expect() on prod paths.
- After modifying canvas.svelte.ts: svelte-check passes, CanvasMode union includes 'process', isModeWired('process') returns true.
- After creating ProcessLayer.svelte + AgentTab.svelte: svelte-check passes, ProcessLayer renders a node per ProcessNode, AgentTab shows scoped journal rows.
- After creating the contract test: cargo nextest run -p elai-contracts j60_process_canvas_contract passes (a) journal derivation, (b) byte-identical re-derive, (c) TD-3 rejection, (d) zero-rows->no-node, (e) op-stream diff, (f) spawn_view reachability.
- After modifying production_agent_rpc.rs: cargo check -p elai-orchestrator compiles clean, spawn_view reuse compiles, no new mutation op added.

### ON_FAIL
- process_canvas.rs fails to compile because FsmState/AgentPool API differs from plan-asserted signatures: CAUSE = stale anchor; COUNTER-MOVE = re-grep the real struct/fn signatures from fsm.rs/agent_pool.rs and adjust.
- svelte-check fails on ProcessLayer.svelte because CanvasMode union was not updated first: CAUSE = ordering; COUNTER-MOVE = modify canvas.svelte.ts BEFORE the svelte components.
- Contract test fails on byte-identical re-derive: CAUSE = non-deterministic ordering or timestamp leak; COUNTER-MOVE = sort journal rows by (seq, ts) and exclude non-deterministic fields from the hash.
- spawn_view reuse fails because the API surface changed: CAUSE = stale anchor at production_agent_rpc.rs:1340; COUNTER-MOVE = re-grep spawn_view signature and adjust the call.
- cargo check fails on production_agent_rpc.rs edit: CAUSE = the edit introduced a type mismatch; COUNTER-MOVE = read the surrounding code context and fix the types.

### FORK
- FORK-1: if the PhaseTransition SSB action_type is per-transition (not per-phase), the ProcessNode binding uses transition grain not phase grain. TRIGGER: grep ssb.rs for PhaseTransition payload shape; if it carries transition-level data, bind per-transition.
- FORK-2: if PREFER-REUSE (no +ProcessViewProjected variant) is chosen, the tab-open beacon reuses CanvasViewEvent. TRIGGER: default = reuse; only add a new variant if the audit finds CanvasViewEvent semantically wrong for tab-open.
- FORK-3: if no runtime agent edits in an isolatable tree pre-I-2, the v1 worktree diff is per-FSM-session not per-agent. TRIGGER: grep for which AgentRole actually produces EditHistoryEvent rows; if only the session-level exists, bind the diff to session.

### RECON_NEEDED
- RECON-1: re-grep ssb.rs for the exact PhaseTransition action_type name (PLAN-asserted as "PhaseTransition"); the executor must verify the literal string before binding ProcessNode to it. CHECK: rg "PhaseTransition" crates/orchestrator/src/ssb/action_type.rs
- RECON-2: re-grep for the real pre-image capture site. The plan's original anchor (file_ops.rs:272/300) is STALE. The real capture is at crates/orchestrator/src/architecture/timetravel.rs:310 (capture_file). CHECK: rg "pub fn capture_file" crates/orchestrator/src/architecture/timetravel.rs
- RECON-3: confirm which AgentRole's worktree the tab renders pre-I-2. CHECK: rg "EditHistoryEvent" crates/orchestrator/src/ to see which agent roles emit edit rows.
- RECON-4: verify CanvasMode union current state before adding 'process'. CHECK: rg "CanvasMode" crates/dashboard-v2/src/lib/state/canvas.svelte.ts

### ABORT
- If the dependency surface (FsmState, AgentPool, InProcessCanvasDispatcher, spawn_view, CanvasViewEvent, PhaseTransition SSB, EditHistoryEvent SSB) is found to be DARK at edit time (not just stale-anchored but genuinely absent), STOP and escalate — the plan's premise is broken.
- If the contract test cannot achieve byte-identical re-derive under ELAI_SEED after 2 attempts (non-deterministic source identified and unfixable), STOP — GD-2 is violated and the design needs owner input.
- If cargo check --workspace fails with errors OUTSIDE the J-60 changeset (pre-existing breakage), STOP and surface — do not fix unrelated code.

### VERIFY
- VERIFY-1: CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai cargo check -p elai-orchestrator --all-targets exit 0 (off-agent: the compile gate via opus_land.py).
- VERIFY-2: CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai cargo nextest run -p elai-contracts j60_process_canvas_contract exit 0 (off-agent: the contract test gate via opus_land.py).
- VERIFY-3: bun run svelte-check (or the dashboard-v2 svelte-check equivalent) exit 0 for the frontend pair (off-agent).
- VERIFY-4: python .claude/tools/production_caller_check.py --files crates/orchestrator/src/process_canvas.rs exit 0 (no DARK symbols; off-agent wire checker).
- VERIFY-5: python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md exit 0 (contract satisfied; off-agent).
- VERIFY-6: rg -c "set_status.*model|llm.*verdict|status.*from_model" crates/orchestrator/src/process_canvas.rs == 0 (TD-3: no LLM-set status; off-agent grep).

### REDTEAM
- ATTACK-1: "the canvas fabricates node status from model output instead of journal rows." DEFENSE: contract test (c) explicitly rejects model-sourced status; TD-3 negative test. The attack FAILS because the contract test asserts a model-sourced status is rejected.
- ATTACK-2: "a zero-evidence agent renders a fabricated node." DEFENSE: contract test (d) asserts an agent with zero journal rows yields no node (R1). The attack FAILS.
- ATTACK-3: "the worktree diff is fabricated from model text, not the op-stream." DEFENSE: contract test (e) asserts the diff source is the op-stream/H-49 pre-image. The attack FAILS.
- ATTACK-4: "auto-expand creeps in, creating node soup." DEFENSE: the deixis-only-descend discipline is enforced by the UI only rendering a node on explicit descend; contract test asserts ascending re-collapses. The attack FAILS.

### INTEGRATION_SMOKE
- After units 3 (process_canvas.rs) + 4 (ProcessLayer.svelte) + 5 (AgentTab.svelte) land: drive the REAL elai entrypoint with a minimal FSM task that spawns >=1 agent_pool worker, open the PROCESS canvas, descend an agent node. ASSERT: the phase_transition + agent-lifecycle rows the run already emits are visible as process nodes; descending opens the agent's tab with scoped journal rows. This is the R10 cross-seam smoke (Rust backend + Svelte frontend + Tauri IPC).
- INTEGRATION_SMOKE: n/a for units 1-2 (canvas.svelte.ts altitude axis + decisions doc are single-file, no cross-seam).

### PROOF_SPINE
PROOF_SPINE: none (observability canvas, not a PS-number feeder; user-acked as a CFW-3 WAVE watch-half falsifier)
