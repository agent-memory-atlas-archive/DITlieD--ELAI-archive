E-43 publish-readiness checklist (2026-05-27)

Per-skill PASS/FAIL grades. Publication requires ALL gates green:
  - SKILL.md frontmatter passes agentskills_validator.py (exit 0)
  - Body <=500 lines (recommended ceiling)
  - eval/positive.jsonl precision >= 0.95 on lexical-match harness
  - mcp-scan whole-directory scan returns 0 findings (RUNPOD-PENDING)
  - License field present (MIT for publishable, internal for internal)

skill                  validator  body<=500  eval_prec>=95  mcp_scan  license  PUBLISH-READY  notes
  anti-spiral          PASS       UNCHECKED  PASS(1.00)     PENDING   MIT      PARTIAL        recall 0.50 (lexical-match strict); RUNPOD-PENDING scan
  oracle               PASS       UNCHECKED  FAIL(0.00)     PENDING   MIT      NO             eval precision 0.0; description tokens do not overlap eval prompts; corpora drift
  prompt-transform     PASS       UNCHECKED  PASS(1.00)     PENDING   MIT      PARTIAL        recall 0.40; RUNPOD-PENDING scan

internal skills (not for external publication; tracked for completeness):
  b-plan-runner        PASS       UNCHECKED  PASS(1.00)     N/A       internal NOT_PUBLIC      internal-only
  dashboard-handoff    PASS       UNCHECKED  PASS(1.00)     N/A       internal NOT_PUBLIC      internal-only
  idea-intake          PASS       UNCHECKED  FAIL(0.89)     N/A       internal NOT_PUBLIC      internal-only
  impact-check         PASS       UNCHECKED  PASS(1.00)     N/A       internal NOT_PUBLIC      internal-only
  orchestrate          PASS       UNCHECKED  FAIL(0.78)     N/A       internal NOT_PUBLIC      internal-only
  planning-workflow    PASS       UNCHECKED  PASS(1.00)     N/A       internal NOT_PUBLIC      internal-only
  rag-tools            PASS       UNCHECKED  FAIL(0.00)     N/A       internal NOT_PUBLIC      eval corpora missing token overlap
  sidecar-protocol     PASS       UNCHECKED  PASS(1.00)     N/A       internal NOT_PUBLIC      internal-only

Notes:
- body<=500 column UNCHECKED in this run because the deterministic body-line counter is a follow-up; auditor verifies at phase-end audit time.
- mcp_scan column PENDING because the snyk-labs/mcp-scan dist-channel verification is the Open Question flagged in workflow/reasoning/E-43-decisions.md DECISION-9. Until a verified tool is wired, the import gate is text-only soft-gate (skill-import-policy.md).
- Soft-gate threshold here uses precision >= 0.95 per the publication readiness checklist; the runtime SSB emit threshold is precision >= 0.90 / recall >= 0.80.
- The lexical-match harness is intentionally strict; oracle's eval corpora describe behaviors via phrases that don't share 4-letter tokens with the description. Tightening the corpora is a follow-up; flagged here so future plan iterations refine the trigger lexicon.

Grounding:
- agentskills.io/specification (frontmatter shape)
- snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub (scan requirement)
- developers.openai.com/blog/eval-skills (precision/recall thresholds)
