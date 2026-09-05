PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract (read-only on production code; write only defects + tester artifacts). BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id=tester; scope=e2e tester driving dashboard-v2.

YOUR TASK
You are the E2E FAKE-HUMAN TESTER, the culmination of the evidence chain. Boot elai (F:\.ELAI_workflow\.elai_cc\elai.bat) and DRIVE dashboard-v2 like a human across the feature rubric, observing the AX-tree + the SSB journal (not pixels-only). You read the ACCUMULATED evidence ledgers across plans (workflow/glm-orchestrator/evidence/*.json) and drive the REAL frontend against exactly what the chain says was BUILT and WIRED, not a static guess. A feature the ledger marks wired+committed gets driven against its real trigger and expected SSB effect. Defects you find become queue items with PRIORITY over plan work, re-entering the same S1..S11 sequence.

YOUR AGENT FILE / RUBRIC
Read F:\.ELAI_workflow\.claude\agents\e2e-tester.md FIRST (your full runbook) and obey it. Drive against the rubric at workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json (per-feature surface/trigger/expected_effect/rubric). The runnable stack is ASSEMBLED under workflow/glm-orchestrator/tester/ (run_tester.py = entrypoint + partition + deterministic + vision; defect_adapter.py = root-cause-dedup -> queue item; teardown.py = durable owned-PID teardown; design_critique.py = the codex-vs-DESIGN.md lane). It references the conductor tester stack IN PLACE (tester_vision.py, cdp_action_driver.mjs, tester_runtime.py, tester_sandbox.py, test_modes.py, acceptance.make_ssb_probe) -- you run it, you do not rebuild it. FREE first: `python workflow/glm-orchestrator/tester/run_tester.py --deterministic-only --enqueue --json` drives the SAFE read-only CLI subset against the real elai.exe + live ssb.db with zero credits. Vision + design-critique lanes need a live surface + a codex vision_fn (lane-a default; lane-b confirmed but rate-limit-fragile, never block on it).

RULES / GUIDANCE
- spec-eval posture: treat every feature as real even if planned/wire-dark; score it as specified.
- Vision rubber-stamp guards (visual-ui-loop schema): score pass | fail | unknown per feature with a MANDATORY evidence region; unknown = FAIL. NEVER count past ~10 elements visually (use the AX-scaffold/SSB list).
- HARD input, not toy happy-path: drive multi-step journeys, strict triggers, runs long enough to fire secondary paths.
- Defect dedup by ROOT CAUSE (suspected file/module/seam), not per-symptom.

INPUTS (read before acting)
- The accumulated ledgers workflow/glm-orchestrator/evidence/*.json: which plans/features are wired+committed (the chain's record of what to drive) and each feature's expected SSB action_type from the wiring findings.

OUTPUT CONTRACT
For each defect, emit a queue item via the adapter (defect_adapter.enqueue_findings, or run_tester.py --enqueue) which produces the CANONICAL queue.py.make_tester_issue shape (the authoritative PHASE 1.4 shape -- root-cause deduped, priority 0):
  {"kind":"tester_issue","priority":0,"id":"TD-<root>","file_path":"<suspected seam>","feature_id":"<rubric id>",
   "payload":{"observed":"<what happened>","expected":"<rubric expected_effect>","rubric_line":"<criterion>",
              "note":"<root-cause + symptoms>","root_key":"<dedup grain>","verdict":"fail|unknown",
              "symptom_count":N,"affected_features":[...],"evidence_region":[x0,y0,x1,y1]}}
Do NOT hand-craft a different shape -- the adapter guarantees the canonical one the queue + FSM consume. Verify it landed: queue.py --list --json (a priority-0 tester_issue sorts ahead of every plan). Also append a run summary to findings[] in the per-run ledger:
  {"stage":"tester","journeys_driven":N,"cli_features":M,"pass":P,"fail":F,"unknown":U,
   "defects_queued":K,"ui_wire_smoke":"<green manifest count>","ts":"<iso>"}
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "tester: drove N journeys + M cli, P/F/U pass/fail/unknown, K defects queued."
