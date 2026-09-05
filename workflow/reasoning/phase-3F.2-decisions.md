Phase 3F.2 decisions

What: Predicate type layer for rules/ crate (6 variants, verdict types, evaluator traits, validation).

Key decisions:
1. Included all 6 predicate types (rules.txt section 3 is authoritative over phase-3F.txt "5" count). GraphReachability as 6th variant with DkgQuerier trait stub.
2. threshold changed to Option f64 for MetricGate. Between operator carries its own low/high; Lt/Gt/Eq require Some. validate() enforces this.
3. from_json() = serde_json::from_str + validate() as canonical deserialization path. Derives Deserialize kept for flexibility; from_json is documented-preferred but not enforced at type level. Tradeoff: enforcing via serde(try_from) adds complexity for marginal safety gain since all DB reads should go through from_json.
4. validate() checks: depth limit (max 8), Not arity (exactly 1 child), MetricGate threshold presence. Recursive for nested composites.
5. No new dependencies. serde_json already in workspace. No postcard, no regex (regex needed in 3F.3, not 3F.2).
6. Serde format: #[serde(tag = "type", rename_all = "snake_case")] for self-describing JSON.

Expected test outcome: 35 unit + 25 contract = 60 green. Schema regression: 16 green. Total 76/76.
Unknowns: DkgQuerier and AstEvaluator are trait stubs only; real implementations in 3F.3.
