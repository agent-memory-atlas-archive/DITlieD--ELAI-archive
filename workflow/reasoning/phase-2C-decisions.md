Phase 2C decisions
Session: s-821352
Date: 2026-04-06

What: per-agent GRC configuration — different budgets, step sequences, grounding constraints, and allowed verification methods per agent role.

Key decisions:
  D1: per-agent transition_overrides (additive-only merge with default allowed_next). Needed because auditor's double-verify requires verify->verify which 2A doesn't allow.
  D2: config validation at load time (5 rules, fail-fast). Catches impossible configs before runtime.
  D3: TOML layering [grc.defaults] + [agents.{role}.grc], field-by-field override. Follows existing underscore convention.
  D4: allowed_methods per role (policy layer, distinct from capability layer in reasoning.txt section 9).
  D5: anti-rubber-stamp enforcement via FINDING: prefix check on auditor Converge.
  D6 (from audit): step whitelist filter — required_steps restricts reachable states via intersection with merged transitions.
  D7 (from audit): GroundingLevel enum must order { Uncertain=0, High=1, Verified=2 } for derived PartialOrd.
  D8 (from audit): agent-side budget enforcement is authoritative in phases 0-3, demoted to advisory in phase 4A.

Expected test outcome: 13 tests covering all 5 agent roles, double-verify, observe-only, anti-rubber-stamp, config validation, grounding level enforcement, allowed methods, TOML layering, transition additive-only.

Unknowns:
  - GroundingLevel enum order in phase 2A output (may need coordination with s-1003192)
  - Exact TOML table structure for [agents.{role}.grc] (need to align with core::Config parser)
  - Whether FINDING_PREFIX is sufficient or a structured FindingReport type is needed (deferred to implementation)

Audit findings resolved: 4 BLOCKs fixed, 7 WARNs addressed. See workflow/reasoning/phase-2C-grounding.md for full grounding log.
