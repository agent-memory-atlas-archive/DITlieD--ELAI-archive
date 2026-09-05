Phase 3G Grounding Log
Date: 2026-04-06
Session: s-1728096

Claims grounded during phase improvement:

1. "skeleton.toml format with goal/inputs/outputs/constraints/depends_on/depended_by/version fields"
   -> GROUNDED: orchestrator.txt L219-226 (exact TOML example with all fields)

2. "2-4k token budget for skeleton"
   -> GROUNDED: orchestrator.txt L215 ("~2-4k tokens, always loaded in Slot 3")
   Note: phase file uses 4096 hard cap, which is the upper bound of the range.

3. "4096 hard cap enforcement"
   -> GROUNDED: orchestrator.txt L608 testable invariant ("Plan skeleton fits within 4k tokens")
   CORRECTION: originally cited implementation-guide.txt L608 which does not exist (file has 233 lines).
   The invariant is in orchestrator.txt L608.

4. "SSB action_type='plan_update' for storage"
   -> GROUNDED: orchestrator.txt L248 ("STORAGE: skeleton + phase details in SSB as action_type='plan_update'")

5. "NOT in RAG/vector search"
   -> GROUNDED: orchestrator.txt L249 ("NOT in RAG/vector search. Plans are structured — use SQLite queries")

6. "3-step contract extraction algorithm"
   -> GROUNDED: orchestrator.txt L234-239 (input contracts, output contracts, shared constraints)

7. "3 coherence checks by auditor"
   -> GROUNDED: orchestrator.txt L241-246 (satisfies contracts, no undeclared interfaces, version incremented)

8. "contract change propagates to dependent phases"
   -> GROUNDED: orchestrator.txt L246 ("Skeleton contract change -> propagate to all dependent phases (flag for re-review)")

9. "skeleton loaded in Slot 3 with active phase detail"
   -> GROUNDED: orchestrator.txt L215 ("always loaded in Slot 3"), L231 ("Loaded into Slot 3 alongside skeleton")

10. "token_count_estimate for allocation, token_count for validation"
    -> GROUNDED: providers.txt section 1, phase-3E.txt TOKEN COUNTING STRATEGY

11. "Kahn's algorithm for topological sort and cycle detection"
    -> VERIFIED: standard CS algorithm, O(V+E), detects cycles as side effect (remaining nodes = cycle)
    Web search confirmed GoalAct (arxiv 2504.16563) uses hierarchical decomposition with dependency tracking.

12. "SSB single-writer guarantees sequential version updates"
    -> GROUNDED: orchestrator.txt L48-49 ("All writes through single-writer tokio::mpsc channel")

13. "IndexMap for ordered phase storage"
    -> VERIFIED: indexmap crate (crates.io) preserves insertion order, O(1) lookup

14. "SQL queries for plan phase retrieval"
    -> GROUNDED: orchestrator.txt L250 ("SELECT * FROM plan_phases WHERE phase_id IN (?, dependencies_of(?), dependents_of(?))")

Improvements added (with evidence):

A. Expanded from 4 vague steps to 7 detailed steps with full improvement blocks
   Evidence: Phase 3E (gold standard) has 7 steps with same block structure

B. Added R24 risk (implicit dependencies missed)
   Evidence: orchestrator.txt L237 explicit constraints field addresses this

C. Added R25 risk (coherence check false positives)
   Evidence: structural vs semantic distinction from orchestrator.txt L241

D. Added R26 risk (concurrent version edits)
   Evidence: mitigated by SSB single-writer (orchestrator.txt L48)

E. Added ContractRef struct with type_signature and invariants
   Evidence: orchestrator.txt L220-225 shows typed contracts (inputs/outputs with signatures)

F. Added ExtractedContracts struct
   Evidence: orchestrator.txt L238-239 describes contracts prepended to detail

G. Added skeleton_version tracking in PhaseDetail for staleness detection
   Evidence: orchestrator.txt L246 implies version comparison needed for propagation

H. Added contract conflict detection (duplicate names, different signatures)
   Evidence: logical necessity when merging contracts from multiple dependency phases

I. Added cycle detection with Kahn's algorithm
   Evidence: DAG property is foundational to dependency graphs (implementation-guide.txt DEPENDENCY GRAPH)

J. Added stub_reference action type (orchestrator.txt L62 assigns to Section 9)
   Evidence: orchestrator.txt L62 ("stub_reference (Section 9)")

K. Changed coherence check 2 from free-text parsing to structured set comparison
   Evidence: free-text Rust parsing is non-deterministic (BLOCK-6 from adversarial audit)

L. Added implemented_contracts and declared_interfaces to PhaseDetail struct
   Evidence: needed for deterministic coherence checks (structured comparison vs text grep)

M. Added version enforcement at write time (WARN-9 from adversarial audit)
   Evidence: version increment should be enforced at write, not detected post-elaboration

N. Clarified storage model: TOML files authoritative, SSB is audit trail
   Evidence: orchestrator.txt L250 SQL is conceptual shorthand; plan_phases table
   does not exist in SSB schema (ssb + ssb_files only)

Grounding corrections from adversarial audit:
- BLOCK-1: "implementation-guide.txt L608" -> corrected to "orchestrator.txt L608"
- BLOCK-5: skeleton.version GROUNDED at orchestrator.txt L245, global_constraints GROUNDED at L237,
  created_at/updated_at marked UNGROUNDED (standard practice)
- BLOCK-5: PhaseDetail.skeleton_version marked UNGROUNDED (implied, not explicit)
- WARN-6: dismissed — providers.txt section 10 exists at L184 (TESTABLE INVARIANTS), 15% at L186

External sources consulted:
- arxiv 2504.16563 (GoalAct): hierarchical planning with dependency tracking, confirms DAG traversal pattern
- BudgetThinker (OpenReview): token-budget-aware reasoning — conceptually aligned but not directly applicable
- Token-Budget-Aware LLM Reasoning (arxiv 2412.18547): dynamic token allocation — confirms estimate-then-exact pattern
