P3G.6 Decision Doc

What: Cycle detection (Kahn's), BFS propagate_changes, split_phase with contract preservation, coherence_check_contract.rs

Key decisions:
- Kahn's algorithm for cycle detection (topological sort based, O(V+E), deterministic via BTreeSet for cycle reporting)
- BFS over depended_by for propagate_changes (not DFS — BFS matches spec's "flagged" semantics, visited HashSet prevents diamond duplication)
- SSB writes in propagate_changes reuse PlanUpdatePayload{plan_hash, phase, step} — phase=flagged_id, step=source_id (no new payload type)
- split_depth: u8 added to PhaseEntry with serde(default) for backward compat. Max depth 2 enforced by SplitDepthExceeded error.
- kahn_cycle_check wired into write_to_file BEFORE version checks — reject invalid DAGs before any disk/SSB write
- No new crate dependencies — all from std (HashMap, HashSet, BTreeSet, VecDeque)
- coherence_check_contract.rs created as separate file (exit criteria requirement), 6 tests covering checks 1-3

Expected test outcomes:
- Chain A->B->C->D: propagate A -> [B,C,D] flagged
- Diamond A->{B,C}->D: propagate A -> [B,C,D] (D once)
- Isolated E: propagate E -> []
- Cycle A->B->C->A: kahn -> Err(CycleDetected{["A","B","C"]})
- Split B(depth=0) into B_1,B_2: B_1.inputs=B.inputs, B_2.outputs=B.outputs, B_1->B_2 internal contract
- Split depth exceeded: split B_1(depth=1) again -> Err(SplitDepthExceeded)

Unknowns: none — all types and SSB API verified from source.
