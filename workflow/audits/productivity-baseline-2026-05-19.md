E-23 productivity baseline (2026-05-19)
========================================

CONTEXT
-------

Per CLAUDE.md HARD RULE grounded-by-default + memory feedback_verify_
semantics: this artifact is a BASELINE, not a counter-measurement to
METR. The METR RCT (arXiv:2507.09089, Becker et al., 16 senior OSS devs,
246 tasks, Feb-Jun 2025, Cursor Pro + Claude Sonnet 3.5/3.7) reports
participants were 19% slower with the AI tool while they felt 20% faster.
The comparison framing in this file is "what is ELAI's measured throughput
on its OWN tree", not "ELAI vs METR" -- paired-task data would be needed
for the latter, and no such data exists today.

REFERENCE POINT (METR, NOT counter-measurement)
-----------------------------------------------

* paper: arXiv:2507.09089 (May 2025 results, July 2025 publication).
* effect: -19% throughput on AI condition vs control.
* felt-faster gap: +20% (subjective speedup illusion).
* sample: 16 senior open-source developers, 246 tasks on 6 real
  repositories.
* tools: Cursor Pro + Claude Sonnet 3.5 / Sonnet 3.7.

This artifact uses METR as the reference frame for the TrustTelemetry
panel's "what would the user actually want to measure" question. We do
not claim ELAI beats this number. We claim only that the harness exposes
the signals the user would need to build a paired-task comparison if
they chose to.

MEASURED NUMBERS (ELAI self-tree, 2026-04-19 to 2026-05-19)
-----------------------------------------------------------

These numbers describe ELAIs development on its OWN tree over the prior
30 days. They are NOT generalizable to user-facing tasks; they are
provided so a future paired comparison can use them as the ELAI side of
the equation.

* commits in last 30 days:                 84
* mean commits per day:                    2.8
* completion-drive session count:          3 (per recent commit log)
* batched plans closed across sessions:    >50 (B/C/D/E across batches)
* schema bumps in window:                  ~20 (SCHEMA_VERSION 150 ish
                                              to 171 across the window)
* full-workspace cargo check status:       GREEN (per activeContext.md
                                              "cargo check --workspace
                                              GREEN as of E-55 close").
* full-workspace cargo test status:        not run in this artifact
                                              (auditor responsibility;
                                              RUNPOD-PENDING tests not
                                              counted).

KNOWN BIASES
------------

* the "completed plan" unit is not normalized for size. one E-N plan
  may take 30 minutes; another takes 4 hours. so commits-per-day is a
  proxy only.
* contract-test scaffolding plans (the .a variants) are typically
  smaller than full-implementation plans, so the bulk of the count is
  scaffolding, not load-bearing code.
* the audit-pass-rate signal would require parsing the auditor reports
  in workflow/sessions/ -- deferred to a follow-up artifact when the
  TrustTelemetryPanel surfaces it directly via SSB rows.

SIGNALS THE PANEL SURFACES
---------------------------

ELAIs TrustTelemetryPanel (E-23 Task 2) renders 5 cards aggregated
from existing SSB rows by `orchestrator::trust_telemetry::aggregate_
session`:

1. tasks_per_hour      -- phase_transition rows into COMPLETE /
                          elapsed-session-hours.
2. findings_per_task   -- auditor finding rows / completed task.
3. revert_rate         -- file_write rows with revert=true / total
                          file_write rows.
4. ungrounded_count    -- mcp_think claim rows with empty grounding.
5. stagnation_dist     -- C-2 / B-22b spinning + oscillation +
                          no_drift + diminishing_returns counts.

The panel is wire-dark by default. emission gated by env var
ELAI_DEVELOPER_DISCOURSE_ENABLE=1. The .elai_cc/README.md "Honest
positioning" section documents this.

FOLLOW-UPS
----------

* paired-task comparison RUNPOD-PENDING. requires a fixed task set
  (e.g. SWE-bench-Verified) run on both ELAI and a Cursor + Sonnet
  baseline. E-21 (SWE-bench-Verified gate) ships this paired comparison
  signal.
* audit-pass-rate timeseries from auditor SSB rows. captured under
  separate plan once the per-level auditor (B-41) ships its
  audit_completed row to SSB. flagged as "Goodhart risk" in
  D-18-decisions.md -- aggregating audit pass rate creates pressure
  to relax auditors which is the wrong direction.

GROUNDING
---------

* METR RCT: arXiv:2507.09089 (Becker et al., July 2025).
* Stack Overflow 2025: survey.stackoverflow.co/2025 (84% AI use,
  29% trust on AI-generated code).
* JetBrains AI Pulse April 2026:
  blog.jetbrains.com/research/2026/04/which-ai-coding-tools-do-
  developers-actually-use-at-work (90% regular use, Claude Code
  91% CSAT / NPS 54).
* memory feedback_verify_semantics: "PROVEN" requires output content
  matching input semantics -- a 511-byte Vite splash is not "ELAI
  built a TODO app". Same standard applied here: this baseline is
  not a comparison.
* CLAUDE.md grounded-by-default: every claim cites a source or is
  labeled Ungrounded. no marketing prose.
