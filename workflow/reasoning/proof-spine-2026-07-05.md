PROOF SPINE — the ruling (2026-07-05)
Origin: spec-eval workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md §5a, adopted by the user
("create the proof spine, add it to our wiring rules/skills/tools"). This is KD-1 (number-before-claim)
lifted from marketing copy to BUILD ORDER at corpus grain. It composes with K-0's A-1..A-4 gating,
GATE M-0, and N-1 (the measurement-spine subsystem); it restates none of them and changes no done-gate.

THE RULING (one sentence)
The corpus's credibility compresses onto six measured numbers; any unbuilt unit that feeds one of them
outranks any unit that does not, and new plan files must declare which (or none, with explicit user ack).

THE SIX SPINE NUMBERS (PS-1..PS-6)
PS-1  SAFETY DELTA — gate-ON-vs-OFF residual-insecure-rate reaching a reviewer.
      Owner: G-6 (`elai bench safedelta`); precursor F-2b (`elai bench seccodeplt` insecure-rate).
      Status at ruling: runner substrate shipped per K/G indexes (seccodeplt_runner, safedelta_runner
      referenced as composable shapes); headline number NOT yet produced. [confirm against live code]
PS-2  COMPOSED GATE-TAX — whole-system added wall-clock + over-block rate, gates-ON vs OFF, vs a
      locked ceiling. Owner: K-4 (`elai bench gate-tax`). Status: PLANNED.
PS-3  HALLUCINATION DELTA — unsupported/contradicted-claim rate, CGG ON vs OFF.
      Owner: L-10 (`elai bench hallucination-delta`). Status: PLANNED (fixture tier first).
PS-4  ROUTING ECONOMICS — $/verified-task, cloud-only vs architect/editor hybrid.
      Owner: K-2 (`elai bench routing-economics`); live cost leg BLOCKS on J-28. Status: PLANNED.
PS-5  CASE-REUSE COMPOUNDING — recurrence-hit-rate + reuse-ON iteration/$ delta ("gets cheaper on
      YOUR repo"). Owner: K-18 (`elai bench case-reuse`). Status: PLANNED (observe-first arm).
PS-6  WIRE-DARK RATE, LONGITUDINAL — the % of shipped units with no production caller, before vs
      after the enforcement stack. Status: PRE-stack ~40-45% (wiring-enforcement.md, ~187 plans);
      2026-06-04 audit 28% SHIPPED_WIRED over 213 plans; needs a dated post-stack re-measure to
      become the first publishable spine number (needs no RunPod, no cloud — cheapest first win).

SUBSTRATE ORDER
N-1 (one runner family + one corpus family + one calibration sink + the outcome-label fold) is the
substrate all six ride; among unbuilt units it goes FIRST. The K-0 literal-first-command (invoke the
shipped A-1 kill-gate) stands unchanged — a kill-switch verdict outranks this ruling.

THE PRECEDENCE RULE (binding, see .claude/rules/proof-spine.md)
PSR-1 Any session choosing "what next" ranks candidates: (0) A-1..A-4 kill-gate invocations,
      (1) N-1 substrate, (2) PS-owner units + their named blockers (J-28 for PS-4), (3) units that
      feed a PS number transitively (declared, not vibes), (4) everything else.
PSR-2 Every NEW plan file carries a PROOF_SPINE: line — `feeds: PS-N <how>` or `none (<class>)`.
      A `none` requires explicit user ack in the authoring session (recorded in the file).
PSR-3 The spine numbers themselves obey KD-1/GD-2/GD-3: measured, journal-recomputable, ablation-
      framed; this ruling never fabricates a number and never discharges any done-gate (advisory
      sequencing layer; wiring-enforcement R1-R10 untouched, gate-manifest untouched).
PSR-4 Layer-0 triage: `python .claude/tools/proof_spine_status.py [--json]` — the advisory
      what-is-measured/what-is-pending read (elai_status_query boundary: never signs off anything).

FIRST THREE MOVES UNDER THIS RULING
1. Run the A-1 kill-gate command (K-0 NEXT STEP, unchanged).
2. PS-6 post-stack re-measure (bn_status_report + production_caller_check sweep over the current
   tree; a dated ledger row here — the cheapest real number, zero new build).
3. N-1, then the PS-owner benches in K-0/GATE-M-0 order.

WHAT THIS RULING DOES NOT DO
- Does not remove/park any unit (removals from the spec-eval §5b remain PROPOSALS awaiting explicit
  user confirmation, per the house rule).
- Does not modify wiring-enforcement.md, the gate manifest, or any frozen artifact.
- Does not create a new letter (K-10 upheld).

Files landed with this ruling: .claude/rules/proof-spine.md (the rule), .claude/tools/
proof_spine_status.py (the Layer-0 tool), a conditional-routing trigger line, and the two
user-directed plan files K-25 + M-6 (both PROOF_SPINE: none, user-acked).

APPENDED 2026-07-17 (source-of-truth state files, owner-authorized): the six numbers now have a
versioned SOURCE OF TRUTH split from this prose ruling. WHAT each measures (runner, artifact pattern,
gate, lane-tag requirement) lives in .elai_cc/benchmarks/registry.json (versioned, the only git-tracked
path; it also subsumes the full feature-bench inventory of the ~24 `elai bench` subcommands + the
e2e-functional local lane). The adjudicated CURRENT numbers + verdicts + lane tags live in
workflow/status/proof-spine-state.json (writer: proof_spine_status.py, which now reads the registry
and refreshes the state file each run instead of re-deriving verdicts from filename heuristics). The
feeder mirrors a derived feature-bench state to workflow/status/benchmarks-state.json and embeds both
`proof_spine` + `benchmarks` keys into workflow/status/elai-status.json for the TEIKOKU core. Verdict
vocab gained GATED (headline blocked on pod/journal/cloud-dollar). Seeded state 2026-07-17:
PS-5 + PS-6 MEASURED; PS-1 GATED(pod); PS-2 RUNNER_PRESENT(journal); PS-4 RUNNER_PRESENT(cloud-dollar);
PS-3 PLANNED. This is an advisory cache layer (PSR-3/PSR-4 boundary unchanged); it discharges no
done-gate and no frozen artifact was touched.
