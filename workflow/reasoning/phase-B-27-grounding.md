GROUNDING SWEEP -- phase B-27
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-27_cloud_mode.md
Date: 2026-05-10
FSM: PLANNING | Session: s-35596

SPEC REFERENCES
cloud-mode-design.txt (FULL) | GROUNDED | file exists; 3 modes at :33-47, provider routing :63-93, context budget :96-128, layer collapse :136-164, privacy controls :199-233
post-phase4-implementation-order.txt D-1..D-4 | GROUNDED | D-1 at :197-225, D-2 at :228-255, D-3 at :258-285, D-4 at :288-315; all deliverables confirmed
reasoning-architecture-design.txt LAYER 5-6 + CLOUD BUDGET ECONOMICS | GROUNDED | Layer 5 :336-401, Layer 6 :404-423, CLOUD BUDGET ECONOMICS :581-624
summary/integration-summary.md | GROUNDED | file exists at docs/thefinalglance/summary/integration-summary.md
crates/providers/src/* | GROUNDED | providers/src/providers/anthropic.rs + openai_compat.rs confirmed present

NOTE: B-27 has no "Spec references:" label; "## Reads" section serves the same function.

CLAIM CLASSIFICATION
3 modes LOCAL/CLOUD/HYBRID per-task selection | GROUNDED | cloud-mode-design.txt:33-47
200K context vs 8-25K local | GROUNDED | cloud-mode-design.txt:99
[context_budget.cloud] 8 slot values | GROUNDED | cloud-mode-design.txt:111-119 (exact slot values match B-27)
Baby step cloud_sonnet=300 cloud_opus=500 LOC | GROUNDED | cloud-mode-design.txt:121-128
CLOUD skips L2-L5 uses L1+L6 | GROUNDED | cloud-mode-design.txt:140-152
HYBRID L1-L4 local L3+ failure single cloud call | GROUNDED | cloud-mode-design.txt:161-164
CLOUD_SPEND SSB action type | GROUNDED | cloud-mode-design.txt:85-88 (spec only; not yet in ssb.rs -- expected for B-27)
ESCALATE_TO_CLOUD SSB action | GROUNDED | cloud-mode-design.txt:194-196, post-phase4:203
Redaction placeholder [REDACTED:type] | GROUNDED | cloud-mode-design.txt:222-223
BYOE uses OpenAI-compatible adapter | GROUNDED | post-phase4:274-276; openai_compat.rs confirmed
Air-gapped disables all external network | GROUNDED | cloud-mode-design.txt:278-281, post-phase4:278-281
warn_at_percent default 80% | GROUNDED | cloud-mode-design.txt:75
L1 failure max 3 retries then escalation_model | GROUNDED | cloud-mode-design.txt:154-159
Cost estimation display "Estimated cost: $X-$Y. Proceed?" | GROUNDED | post-phase4 D-4:305-306
MONITOR window shows running spend total "$0.00 if zero" | MISMATCH | post-phase4 D-4:308 says "Show '$0.00' only when spend > 0" (i.e., hide when zero). B-27 inverts this.
Cost formula: phase_count * avg_tokens * pricing | MISMATCH | post-phase4 D-4:303-305 formula is sum(steps*avg_tokens_per_step*input_price + output_tokens*output_price) -- B-27 drops input/output split and per-step breakdown
Dashboard CloudPanel provider/model UI spend gauges redaction stats | UNGROUNDED | neither cloud-mode-design.txt nor post-phase4 name "CloudPanel"; D-4:307 only mentions MONITOR window cost display

ACTIONS REQUIRED
- MISMATCH Task 18: "$0.00 if zero" contradicts post-phase4-implementation-order.txt:308 which says show only when spend > 0. Correct B-27 Task 18 to "show spend only when > 0; suppress $0.00 at zero" before TESTING_FIRST.
- MISMATCH Task 17: cost formula "phase_count * avg_tokens * pricing" contradicts D-4:303-305 formula (sum over phases with separate input/output token pricing). Correct B-27 Task 17 to match D-4 formula or explicitly note simplification is intentional.
- UNGROUNDED Task 20: "CloudPanel" component with provider/model selection UI, spend gauges, redaction stats, air-gapped toggle -- no spec anchor. Either add spec citation (if a dashboard-design.txt section covers this) or mark as design decision not spec-derived and add to WIRING_CONTRACT as new deliverable needing dashboard-design.txt coverage.

VALIDATION WARNINGS
B-27 uses "## Reads" heading not "Spec references:" label -- spec_link_check.py may miss these references; manual sweep substituted.
MODE SELECTION section (cloud-mode-design.txt:38-39) internally contradicts REASONING LAYER COLLAPSE section (:140-152) on which layers run in CLOUD mode. B-27 follows the REASONING LAYER COLLAPSE section (authoritative) -- not a B-27 error but a spec inconsistency to note.
