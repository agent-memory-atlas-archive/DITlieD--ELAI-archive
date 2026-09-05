Phase 2C improvement grounding log
Session: s-821352
Date: 2026-04-06

Claims grounded:
  auditor double-verify -> reasoning.txt:97 (required_steps=[observe,hypothesize,verify,verify])
  context-compiler observe-only -> reasoning.txt:103-106
  executor budget 8/1200 -> reasoning.txt:89-91
  memory-writer budget 2/400 -> reasoning.txt:115-116
  global defaults in elai_cc.toml -> reasoning.txt:38
  [agents.{role}] TOML pattern -> grep: 5 specs use it (orchestrator, providers, tools, prompt-architecture)
  anti-rubber-stamp >= 1 issue -> reasoning.txt:99
  per-agent FSM sub-states -> reasoning.txt:118-120
  AgentGrcConfig 4 fields -> reasoning.txt:186
  orchestrator budget enforcement -> orchestrator.txt:440-444
  verify->verify not in default allowed_next -> phase 2A design (GrcState::allowed_next has no Verify->Verify)
  fail-fast config validation -> BudgetMLAgent (arxiv:2411.07464)
  insufficient verification failure mode -> arxiv:2603.26993
  role-based agent config pattern -> MetaGPT, CrewAI (web search confirmed)

Ungrounded (deferred to measurement):
  <1ms config load overhead -> will verify in step 2C.5
  <10% per-step routing overhead -> will verify in step 2C.5

Spec gaps identified:
  1. reasoning.txt:186 AgentGrcConfig needs allowed_methods + transition_overrides fields
  2. reasoning.txt section 11 missing anti-rubber-stamp invariant
  3. reasoning.txt GrcError missing InsufficientFindings + ConfigValidation variants

Design decisions:
  D1: per-agent transition_overrides (additive-only merge with default allowed_next)
  D2: config validation at load time (5 rules, fail-fast)
  D3: TOML layering ([grc.defaults] + [agents.{role}.grc], field-by-field override)
  D4: allowed_methods per role (Vec<VerificationMethod> filter)
  D5: anti-rubber-stamp enforcement (FINDING: prefix check on auditor Converge)
