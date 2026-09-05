# I-6: anchor pane slot — immutable privileged-last context slot that defeats lost-in-the-middle drift

- Track 13 — EVIDENCE-RANKED SEARCH | lever STEERING | grounding STRONG | nature BUILD | effort S | WAVE 2

- Premise: on long-context tasks the model's effective attention to the task specification degrades as the
  context fills. An immutable privileged slot appended LAST (after the final cache breakpoint) carrying the
  TaskSpec, unresolved claims, and active constraints exploits recency bias constructively. The model must
  emit a short acknowledgement tag reflecting the active constraint before generating — a cheap, mechanically
  enforceable attention anchor. Gap confirmed: ContextSlot has 9 slots, none is a privileged-last anchor (IN-3).

## Steal/build
lost-in-the-middle defense: add ContextSlot::AnchorPane as a LAST slot — placed after the final cache
breakpoint so the prefix stability invariant (HD-2) is preserved. The slot carries: the original TaskSpec
bytes (frozen), the set of unresolved claims from I-17/I-20 verification passes, and the active constraint
list. Before generating, the model must open its reply with a short acknowledgement tag that reflects the
highest-priority active constraint; the supervisor validates the tag's presence (I-1 seam). This is a
mechanical recency-bias lever, not a persona or prompt instruction. COMPOSES H-31's lazy-ALTER + slot-add
pattern and H-2's breakpoint discipline. EXCLUDED: rebalancing existing slots to make room (the slot is
appended LAST without displacing others); adding a prompt instruction telling the model "remember the task"
(prompt-governance, not structural); implementing a new context serialization layer (I-6 adds ONE slot to
the existing ContextSlot enum, nothing more). I-6 does NOT add a new action_type — it reuses the existing
ContextReduction payload family per the I-0 SSB delta table (GD-1).

## Existing ground truth
- IS-6: context_budget.rs ContextSlot enum :1079 (nine slots confirmed live; ContextSlot::Memory added at
  H-31 per :10 note) + update_breakpoints :1507 (blake3 prefix stability) + adaptive_reduce :1551. I-6 adds
  the tenth slot as ::AnchorPane LAST variant; prefix stability assert continues to hold.
- IN-3: no AnchorPane / privileged-last slot in the enum confirmed absent — I-6 is a genuine BUILD.
- H-2 breakpoint discipline: PLAN-asserted (Wave-2 predecessor); executor re-greps per W-7. The slot must
  land after the final breakpoint; the bp.valid check must still pass.
- H-31 Memory slot precedent (lazy-ALTER + slot-add pattern): PLAN-asserted; executor re-greps per W-7.
  I-6 mirrors this pattern exactly.
- I-1 supervisor seam (Wave-2): the acknowledgement-tag validation lives on the supervisor plane; I-6
  depends on I-1 for enforcement (Wave-2 dependency within the same wave — ordering within Wave 2 matters).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- ContextSlot::AnchorPane variant added to the enum (context_budget.rs)
- slot assembly: serialize TaskSpec + unresolved-claim list + active constraints into the AnchorPane block,
  appended after the final breakpoint
- acknowledgement-tag contract: the supervisor (I-1) validates presence of the tag in the first reply token
  window; missing tag -> retry prompt with an explicit clarification (not a hard error, but logged)
- ContextReduction payload reuse: no new SSB action_type; the anchor presence is a prompt fact (I-0 table)
- decision doc workflow/reasoning/I-6-decisions.md
DEFER (Tier-2 / gated):
- adaptive constraint prioritisation (rank which constraints appear in the anchor by recent miss-rate) —
  post >=3 measured constraint-adherence runs showing the simpler fixed-list is insufficient
- constraint-adherence cockpit tile — same GD-4 bound gating

## Integration
Existing seam: context_budget.rs ContextSlot enum + assembly function (add the ::AnchorPane variant and
the serialization branch at the assembly callsite — the H-31 lazy-ALTER pattern).
No new crate. No new module file. The change is a targeted enum extension + the assembly hook.
frontend_pair: N/A — the anchor pane is a prompt-internal construct; there is no dashboard surface for it.
W-6 justified: the anchor's effect is observable only through constraint-adherence measurement on fixtures,
not through a live dashboard tile. Justified.

## WIRING_CONTRACT
produces_symbols:
  - context_budget.rs::ContextSlot::AnchorPane (the new enum variant)
  - context_budget.rs::assemble_context (the anchor serialization branch added here)
consumes_symbols:
  - context_budget.rs::update_breakpoints (why: assert bp.valid still holds after LAST slot — GD-1 IS-6)
  - context_budget.rs::ContextSlot::Memory pattern (why: mirror the H-31 lazy-ALTER slot-add precedent — GD-1)
  - I-1 supervisor gate (why: acknowledgement-tag validation lives there — GD-1)
reachable_from:
  - task_runner.rs context assembly path -> assemble_context -> ContextSlot::AnchorPane serialization
    [PRODUCTION task-runner context assembly]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO new SSB action_type for anchor presence — reuse ContextReduction payload family (I-0 SSB table
    explicit rule; GD-1 compose-not-restate)
  - NO rebalancing or displacing existing slots to make room for AnchorPane (append-LAST only)
  - NO prompt instruction ("remember the task") as a substitute for the structural slot (prompt-governance
    is not a mechanical anchor)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated measurements
frontend_pair:
  N/A — anchor pane is prompt-internal; no live dashboard surface. W-6 justified. Justified.
observable_effect:
  trigger: "cargo run --bin elai -- run --task fixtures/search/long_context_25k.toml"
  expect_ssb: "NO new action_type row (anchor presence is a prompt fact; ContextReduction family reused
    per GD-1); the assembled-context journal entry shows the final block is the AnchorPane payload
    (grep 'anchor_pane' in the journaled request bytes); bp.valid assert passes (HD-2)"
  expect_artifact: "the reply's first token window contains the acknowledgement tag (verified from the
    journaled response); constraint-adherence on the seeded long-context fixture is measured (not asserted)
    vs the --no-anchor control arm (ID-5); the --no-anchor broken arm produces a reply that opens WITHOUT
    the tag and FAILS the tag-presence check (R2)"

## Grounding
- IN-3 (I-GROUNDING-MAP): no AnchorPane / privileged-last slot confirmed absent — genuine BUILD.
- IS-6 (I-GROUNDING-MAP): context_budget.rs ContextSlot :1079 + update_breakpoints :1507 — the seams.
- H-31 Memory slot precedent PLAN-asserted; executor re-greps per W-7.
- H-2 breakpoint discipline PLAN-asserted; executor re-greps per W-7.
- I-0 SSB delta table (I-6 entry): "NO new action_type — reuse ContextReduction payload family (GD-1);
  anchor presence is a prompt fact."
- HD-2 (H-0_INDEX): nothing busts the byte-stable cache prefix — a cache-bust is a test failure; the LAST
  placement is exactly the HD-2-safe position.
- CL-1 (H-0_INDEX): context-lifecycle regime awareness binds I-6 per I-0 doctrine table.
- ID-5 (I-0_INDEX): local baseline before paper threshold — constraint-adherence delta is measured on ELAI
  fixtures, not imported from a lost-in-the-middle paper number.
- arXiv:2307.03172 (Lost in the Middle) if cited in the decision doc: WebSearch-confirm before headline.
- SSB: NO new action_type — reuse ContextReduction payload family — baseline unchanged (221/532 as of
  2026-06-10; ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- Phase 3E / H-2 ContextSlot allocator (shipped/Wave-2 plan): adds the AnchorPane slot without rebalancing
  others — do NOT re-implement the slot allocation or breakpoint logic.
- H-31 Memory slot (shipped/Wave-2 plan): the lazy-ALTER + slot-add pattern to mirror exactly.
- I-1 supervisor enforcement plane (Wave 2): the acknowledgement-tag validation enforcer — compose, not
  re-implement gate logic.

End of I-6.
