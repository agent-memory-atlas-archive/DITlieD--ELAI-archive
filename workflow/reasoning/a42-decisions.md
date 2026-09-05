A-42 TOKEN RECLAMATION TECHNIQUES

What: 5 token reclamation techniques added to context_budget.rs (formatting removal, output fmt stub, path abbreviation, import collapsing, deduplication). Output postprocessor gains restore_formatting config field. 7 contract tests added to context_budget_contract.rs.

Key decision: apply_token_reclamation is a standalone public function, not wired into compute_with_profile (which does budget math only). The assembler (build_dispatch_context, future phase) will call it at assembly time. This keeps budget allocation and content transformation separate.

Expected test outcome: 865+ lib tests pass, 28 reclamation-specific tests green, 7 contract tests green.

Unknowns:
- T2 (output formatting restoration) is stub-only due to tree-sitter dep cycle risk. Full implementation deferred to dep audit.
- extract_definition_symbol does not handle pub(crate)/pub(super) visibility prefixes. Documented, not blocking.
- Path abbreviation uses byte length for threshold, assumes ASCII paths. Documented.
