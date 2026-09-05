J-60 decisions — layered observability canvas (grill-resolved 2026-06-20)

what: the watch half of a layered (process/system/script) composition canvas. process layer = ELAI's own FSM + agent_pool workers projected as journal-derived nodes; per-agent tab = descend (semantic zoom) into an agent's process node, showing its scoped slice + worktree.

key decisions (from the grill, workflow handoff to this spec):
- v1 primary job = OBSERVABILITY (read). redirect is J-61. because: emit rides shipped substrate (phase_transition WIRED, H-10 shell); read-value precedes write-value; cheapest falsifier (do people watch it) before the expensive bet (do they steer it).
- node = all three layers on one surface, altitude/semantic-zoom switched, NOT three graphs.
- authorship = LLM emits (projects journal), user navigates only (no mutation this unit).
- per-agent tab = a filtered VIEW of the one journal (forced by GD-2), never first-class state.
- tab = descend: opening an agent tab IS zooming its process node — one navigation primitive (unifies the per-agent-tab memo with altitude legibility).
- per-agent worktree view = the FULL isolated worktree (spec-eval as-specified). I-2 (COW worktree) is a recorded BUILD DEPENDENCY, not a design compromise. until I-2: diff derived from the H-49 journaled op-stream (pre-image at file_ops.rs:272/300). when I-2 lands the diff source swaps, the UI does not.
- deixis-only descend (no auto-expand) — the anti-node-soup discipline vs the Langflow plateau.

invariant: journal is the store; canvas re-derives byte-identically (GD-2); no parallel state; no canvas-side runtime; node status from journal rows only (TD-3); worktree = real working filesystem the canvas projects (not a no-parallel-state violation).

expected test outcome: process nodes derive from a fixture phase_transition + agent-lifecycle journal; re-derive byte-identical under ELAI_SEED; model-sourced status rejected; zero-row agent yields no node; worktree diff from op-stream not model text; tab-open reaches spawn_view production path.

unknowns: which AgentRole's worktree the tab renders pre-I-2 (executor not auditor); phase_transition action_type grain (per-transition vs per-phase); whether the optional ProcessViewProjected beacon earns its golden-lock cost (default prefer-reuse CanvasViewEvent).
