D-24 — Cross-Domain Canary Regression Benchmark Fixtures

Schema (per file, JSON)

  id: stable id ("cr_NNN").
  domain: human-readable domain tag (code | math | reasoning | planning |
    tool_use). Not used for classification; present for human review.
  family: one of "CodeFix" | "MathProof" | "ProseReason" |
    "PlanningDecomposition" | "ToolUse". Drives the D-7 CanaryFamily
    mapping inside the runner.
  expected_class: "Harmful" | "Helpful" | "Neutral". The runner uses
    this to bucket the row into the 4-cell confusion matrix
    (true_block / false_block / true_pass / false_pass).
  pre_baseline_score: target-family pass-rate BEFORE the mutation
    (range [0.0, 1.0]).
  post_change_score: target-family pass-rate AFTER the mutation
    (range [0.0, 1.0]).
  delta_pct: post - pre, persisted for human review. The runner
    re-derives this from the two score fields.
  expected_verdict: "regression_blocked" iff expected_class == Harmful,
    else "pass". Persisted for fixture-authoring clarity.
  provenance: "CanaryCorpus" OR
    {"ElaiSynthetic": {"author": "<reviewer-id>"}}.
  notes: free-form authoring notes.

Composition (100 fixtures total)

  Class distribution:
    20 Harmful (canary MUST block) — strong negative deltas
      (pre 0.80-0.92, post 0.30-0.55).
    50 Helpful (canary MUST NOT block) — strong positive deltas
      (pre 0.40-0.65, post 0.78-0.95).
    30 Neutral (canary MUST NOT block) — near-zero deltas within
      noise floor (pre 0.55-0.75, post +/-1pp of pre).

  Provenance:
    10 CanaryCorpus (2 Harmful + 5 Helpful + 3 Neutral). Adapted
      from D-7's CanaryFamily reference patterns.
    90 ElaiSynthetic (18 Harmful + 45 Helpful + 27 Neutral).
      Hand-authored by session s-25120 reviewers; author field
      carries the session id so the corpus can be re-audited
      against the rubric.

  Family mix:
    All five D-7 CanaryFamily variants represented across the corpus.
    Distribution is approximately uniform across families within each
    class (the generator round-robins uniformly via seeded random).

Gating thresholds (matched by the runner defaults)

  harmful_block_floor    = 20   (block all 20 harmful fixtures)
  helpful_block_ceiling  = 5    (<= 10% false-block rate on helpful arm)
  neutral_block_ceiling  = 2    (<= ~6.6% false-block rate on neutral arm)

Verdict logic

  Pass iff harmful_blocked >= harmful_block_floor AND
          helpful_blocked <= helpful_block_ceiling AND
          neutral_blocked <= neutral_block_ceiling.
  Fail otherwise.

ELAI calibration disclosure

  NanoFlux (arXiv:2509.23252) provides empirical motivation for the
  5-family cross-domain canary mechanism but has NO canary-mechanism
  implementation in the paper. The harmful_block_floor /
  helpful_block_ceiling / neutral_block_ceiling thresholds are ELAI
  calibration values chosen to match the bundled corpus distribution
  (20/50/30 split). They are NOT paper-anchored — re-tuning the
  thresholds requires re-tuning the corpus distribution in lockstep.

Anti-mock-data discipline

  Every row has an explicit Provenance ("CanaryCorpus" or
  "ElaiSynthetic { author }"). Auto-generated rows with no author
  attribution are BLOCKED at fixture authoring time per CLAUDE.md
  stub_ban. The 90 ElaiSynthetic rows in this corpus all carry
  author = "s-25120-D-24-author" so they can be re-audited.

Regeneration

  Re-run the generator script (kept inline in
  workflow/reasoning/D-24-decisions.md DECISION-10) with the same
  seed (424242) to reproduce these fixtures byte-identically. The
  runner's fixture_blake3 will match if and only if the file
  contents + file ordering are unchanged.
