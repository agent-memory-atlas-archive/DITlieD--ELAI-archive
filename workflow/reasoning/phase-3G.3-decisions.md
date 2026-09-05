Phase 3G.3: extract_contracts decisions

What: PlanSkeleton::extract_contracts(phase_id) -> Result<ExtractedContracts, PlanSkeletonError>
Collects input/output contracts from dependency graph and merges constraints.

Key decisions:
1. IndexMap<String, (String, String)> for seen_inputs/seen_outputs: O(1) lookup for conflict detection while preserving insertion order. IndexMap already a dependency from 3G.1.
2. Separate seen_inputs/seen_outputs maps (not shared): input contracts come from depends_on outputs, output contracts from depended_by inputs. Different namespaces — a contract name can appear in both without conflict.
3. Invariants excluded from conflict detection: spec says "same name + same type_signature = not a conflict." Invariants may differ between providers of the same contract. If this becomes a problem, 3G.5 coherence checks can handle invariant merging. Documented as known limitation.
4. Clone on dedup insert: ContractRef must be owned in the output Vec. Clone cost is negligible (small strings).

Expected test outcome: 22 lib + 10 contract = 32 GREEN, 0 warnings.
Unknowns: invariant merging strategy deferred to 3G.5.
