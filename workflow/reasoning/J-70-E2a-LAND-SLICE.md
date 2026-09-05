# J-70 E2a LAND SLICE — product-only residual (auditor financial independence)

This land is the PRODUCT half of J-70. Harness halves E1 and E2b are already installed
on disk under elai/.claude/ (gitignored by design; R8 re-lock done) and are NOT in this
git changeset. Do not REJECT for their absence from the .elai_cc diff.

## E1 STATUS (harness, already on disk — not in this diff)
- .claude/gate-manifest.txt: every frozen-gate row has TAB fields objective/technique/evidence/satisfaction
- .claude/tools/gate_freeze_check.py: --trace matrix + 4-field schema FAIL (exit 3) + re-locked
- Off-agent evidence: conductor/opus_walk/evidence/J-70/E1-landed.md, E1-trace-green.txt, E1-trace-red.txt
- Verified live: `python3 .claude/tools/gate_freeze_check.py --trace` OK (61 files)

## E2b STATUS (harness, already on disk — not in this diff)
- .claude/rules/orchestrate-protocol.md: H-10 SAFETY-CRITICAL CARVE-OUT (J-70 managerial independence)
- confirm-safety-critical-audit-skip required for audit-skip on frozen-gate / Supervisor-plane / P10 paths
- Off-agent evidence: conductor/opus_walk/evidence/J-70/E2b-exec.md

## THIS LAND — E2a only (product, .elai_cc)
produces_symbols:
- AuditorBudgetReserve in crates/reasoning/src/types.rs (hard-reserved auditor budget pool)
- AgentPool.auditor_budget_reserve field + try_reallocate_from_auditor_reserve (ALWAYS Err on non-zero)
- production caller in spawn_inner for every non-auditor role: attempts a real non-zero raid amount
  and must receive Err (financial independence seal)

stub_ban:
- NO advisory-only number the orchestrator can override
- NO theater: production path must pass non-zero (max_calls.max(1), max_chars.max(1)); refuse_raid(0,0)=Ok no-op

observable_effect:
- cargo test -p elai-orchestrator --lib -- auditor_reserve: 3 tests pass
  (refuses_orchestrator_raid, refuse_raid_zero_is_noop, charge_ok)
- try_reallocate_from_auditor_reserve is reachable from spawn_inner (production)

## APPROVE when
- Diff is only the three files under .elai_cc for E2a
- Gates green (cargo test auditor_reserve + cargo check)
- E1/E2b harness absence from the git diff is EXPECTED and not grounds for REJECT
