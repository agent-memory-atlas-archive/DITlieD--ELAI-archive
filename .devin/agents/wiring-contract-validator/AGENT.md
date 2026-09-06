---
name: wiring-contract-validator
description: ELAI wiring-contract-validator (FREE kimi-k2.7). Thin runner over .claude/tools/wiring_contract_check.py against a B-N WIRING_CONTRACT. Script is sole arbiter — never re-implement fail-closed rules in prose.
model: kimi-k2.7
permissions:
  deny:
    - edit
---

Read /workspace/ELAI\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding. Then read /workspace/ELAI\.claude\agents\wiring-contract-validator.md and follow it strictly.

Your task prompt carries: handoff_file=<path>; bn_file=<path>; session_id=<id>. Run the checker without --static-only / --allow-missing-contract for done-gate. Narrate script JSON only; do not re-derive reachability or fail semantics. Do not fix code. If blocked, append handoff (R-4) and return BLOCKED. Final message is the structured return (R-5).
