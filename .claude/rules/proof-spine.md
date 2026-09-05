Proof spine (HARD RULE, sequencing layer — advisory to gates, binding to planning)

Loaded by: any session choosing what to build next, prioritizing the backlog, authoring or amending
plan files, or asked "what should we work on / is X worth building". Canonical ruling + the PS table:
workflow/reasoning/proof-spine-2026-07-05.md. This rule is KD-1 (number-before-claim) applied to
BUILD ORDER: the corpus's credibility compresses onto six measured numbers; units that produce them
outrank everything else.

THE SIX NUMBERS (owners in parentheses; full rows in the ruling doc)
PS-1 safety delta (G-6 safedelta; precursor F-2b seccodeplt)
PS-2 composed gate-tax vs locked ceiling (K-4)
PS-3 hallucination delta, CGG ON/OFF (L-10)
PS-4 routing economics $/verified-task (K-2; live leg blocks on J-28)
PS-5 case-reuse compounding (K-18)
PS-6 wire-dark rate, longitudinal pre/post-stack (no RunPod needed — cheapest first win)

THE RULES
PSR-1 PRECEDENCE when choosing work: (0) A-1..A-4 kill-gate invocations (K-0 order, unchanged) >
  (1) N-1 measurement substrate > (2) PS-owner units + their named blockers > (3) units feeding a
  PS number transitively (the feed must be DECLARED, not vibes) > (4) everything else. Ties break
  toward the cheaper number (PS-6 first — it needs zero new build).
PSR-2 DECLARATION: every NEW plan file carries a `PROOF_SPINE:` line — `feeds: PS-N <one clause how>`
  or `none (<class>)`. A `none` requires explicit user ack in the authoring session, recorded in the
  file. (Precedents: K-25 + M-6, both none, user-acked 2026-07-05.)
PSR-3 BOUNDARY: this layer is sequencing, never verification. It cannot discharge, weaken, or skip
  any done-gate; wiring-enforcement R1-R10, the gate manifest, and GATE M-0 are untouched. The spine
  numbers themselves stay KD-1/GD-2/GD-3 (measured, journal-recomputable, ablation-framed) — this
  rule never asserts a number that has not landed.
PSR-4 TRIAGE TOOL: `python .claude/tools/proof_spine_status.py [--json]` is the Layer-0 advisory
  read (which PS numbers are MEASURED / RUNNER_PRESENT / GATED / PLANNED). WHAT each number measures
  is the versioned registry `.elai_cc/benchmarks/registry.json` (source of truth); the adjudicated
  current numbers + verdicts + lane tags live in `workflow/status/proof-spine-state.json` (the tool's
  single writer; it refreshes live_evidence each run and preserves the adjudicated value/verdict).
  `python .claude/tools/elai_status_query.py --spine` surfaces the same six from the elai-status
  snapshot. Same boundary as elai_status_query.py: an advisory cache for planning; it never signs
  off a changeset, and citing it in an audit PASS is a violation.
PSR-5 SELF-CHECK before proposing any new unit or letter: "which PS number does this feed?" If the
  honest answer is none and the user has not acked a spine-exempt class (discipline / distribution /
  depreciation), do not propose it — propose the blocking spine work instead.
