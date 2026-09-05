# {FEATURE NAME}

- **ELAI ids:** {phase-2A / D-10 / J-83 / ...}
- **Spec status (as designed):** WIRED | PARTIAL | DARK | PLANNED
- **Crate / module (spec):** {e.g. crates/reasoning, crates/orchestrator/src/belief_state.rs}
- **Purpose of this file:** enough architecture to rebuild this organ in another harness. Not a status ledger.

## 1. What it is
Plain definition. Correct technical name first. One paragraph.

## 2. Why it exists
What problem it solves in the ELAI trust spine. What breaks if it is missing or faked.

## 3. How it works (runtime)
Numbered steps from trigger to commit/abstain/fail. Include:
- who calls it
- what input it takes
- what it does deterministically vs what the model may generate
- when it commits, abstains, or fails closed
- what artifact or journal row it leaves

## 4. Types, enums, objects
Name every type the spec defines for this feature. Fields, allowed values, invariants.

## 5. Control flow / state machine
States, allowed transitions, forced transitions, who owns the loop.

## 6. Grounding, receipts, gates
How claims get evidence. Selector hierarchy if any. What is NEVER allowed as a confidence signal.

## 7. Budgets, limits, ceilings
Numeric defaults from the spec. Per-topic vs per-call vs per-role. What happens on overflow.

## 8. Roles and neighbors
Which agent roles use it. What it consumes. What it produces. What must not be forked.

## 9. Config / flags / defaults
Flags, locked keys, default-OFF rules, observe-only-first rules.

## 10. Implementation map
Crate, file, symbol, trait, IPC message. If the spec names them, copy them.

## 11. Journal / SSB events
Event names, payload fields, schema notes from the spec.

## 12. Tests and observables
Named tests, adversarial cases, measurable effects, done-gates.

## 13. Rebuild contract
Bullet list: what another harness MUST implement to have this feature, not a cousin.

## 14. Non-goals / banned designs
Explicit rejections from the spec (e.g. LLM self-eval, MCTS-first, window-as-tier).

## 15. Source citations
Every factual claim cites `path:line` or `path (section)`. Prefer original plan/phase files over summaries.

## 16. Gaps
What the spec leaves unspecified. Label UNCERTAIN. Do not invent.
