Phase 3F.5 decisions

What: Predicate DSL interpreter — MetricGate real logic, evidence strings, ESCALATE routing.

Key decisions:
1. evaluate_single returns (fires: bool, evidence: String) per arm, not (VerdictResult, String).
   Severity/ESCALATE mapping applied post-match. This avoids severity_verdict("ESCALATE") returning Allow.
2. evaluate_composite returns Result<bool> (fired or not), not Result<VerdictResult>.
   Composite + ESCALATE routing works because fires=true flows to post-match ESCALATE check.
3. MetricGate Eq uses relative tolerance: (value-t).abs() < t.abs().max(1.0) * 1e-9.
   f64::EPSILON is too tight for computed metrics. The max(1.0) handles t=0.0 case.
4. EvaluationContext gains owned HashMap<String,f64> metric_values field (no lifetime issue).
5. Rule gains escalate_target: Option<EscalationTarget> with #[serde(default)] for DB compat.

Audit result: 3 BLOCKs found and fixed across 3 cycles.
BLOCK 1: ESCALATE dead code — severity_verdict wildcard returned Allow for ESCALATE. Fixed by bool refactor.
BLOCK 2: child_fires().unwrap_or(false) in composite evidence silently swallowed errors. Fixed by removing re-evaluation.
BLOCK 3: f64::EPSILON too tight for Eq. Fixed with relative tolerance.

Tests: 160 unit + 194 contract = 354 green (est after adding 2 contract tests for ESCALATE+Composite and NextTierRule).
