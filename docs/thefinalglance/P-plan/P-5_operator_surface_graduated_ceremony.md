# P-5: the non-coder operator surface + graduated ceremony — the trust spine scales DOWN, a spell-card generator is not a trading strategy

STATUS: PLANNED (WAVE 1; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — config + render, ~90% composition)

- Track: PERSONAL-INCARNATION | lever: graduated-ceremony-profile | source finding: GAP-5 (answers T1 build-for-one, for the non-coder) | grounding: MODERATE (index-derived, PLAN-asserted) | nature: ~90% composition | effort: M | WAVE 1

- Premise: the corpus's surfaces are expert-dense (drawers, SSB rows, gate verdicts). The thread's build-for-one demand includes Hapiro's "absolutely 0 understanding of coding". Two things are missing: (a) FLEET-GRAIN plain-language operation ("my wardrobe tool broke" -> P-1 diagnosis rendered as a sentence + one gated repair button); (b) GRADUATED CEREMONY — the trust spine must scale DOWN: a D&D spell-card generator does not need the same gate tax as a trading strategy. K-5 risk-tiered gating + K-4's composed gate-tax ceiling are the mechanism; what is missing is an explicit PERSONAL/TOY profile where gates run OBSERVE-ONLY, receipts collapse to one line, and NOTHING is silently skipped. If this tier is not real, the magic-box crowd bounces off ELAI onto simpler harnesses and the whole thesis is forfeit.

## Steal/build
A `CeremonyProfile { tier: toy|personal|project|critical, gate_mode: observe_only|enforce, receipt_verbosity: one_line|full, autonomy }` — a DECLARED, JOURNALED profile (W-2/R1 posture preserved: the tier is chosen and recorded, ceremony is reduced, evidence is NEVER fabricated or silently skipped). K-5 already tiers gating by risk; K-4 already ceilings the composed gate tax — P-5 adds the toy/personal PROFILE on that engine (not a new gate engine). The operator surface is N-7 human-render pointed at the P-1 fleet grain: a red tick renders as ONE sentence + one gated repair button (the repair still passes the real gates; only the CEREMONY around it collapses). At the toy tier, gates run observe-only (they still RUN and journal; they just don't block), receipts collapse to one line.
EXCLUDED: a tier that SKIPS a gate silently (observe-only still runs + journals — the difference is block-vs-warn, never run-vs-not-run; A-3 falsifier posture); fabricated/collapsed-away evidence (the one-line receipt POINTS at the full journal, never replaces it); a second gate engine (ride K-5/K-4 — A-1); a loosening that isn't a declared journaled profile choice (P-7's asymmetry rule: never loosened by vibe).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- K-5 risk-tiered gating already varies gate strictness by risk tier — P-5 adds a toy/personal tier profile to it.
- K-4 measures + ceilings the composed gate tax — the toy-tier tax must be MEASURED near-zero (A-3 falsifier).
- N-7 is the human-render verdict field (render targets today are PR/brief/canvas — professional grains) — P-5 adds a fleet-grain plain-language target.
- P-1 owns the fleet + the red-tick diagnosis P-5 renders; J-32/J-35 front-door + L-14 one-verb build are the non-coder entry substrates.
- GD-4 observe-first + W-2 kill-switch-not-enable posture govern any gate defaulting on.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the CeremonyProfile type (declared, journaled) + the toy/personal profile wiring into K-5's gate-mode selection (observe-only at toy) + K-4 gate-tax measurement at the toy tier + the one-line receipt collapse + the N-7 fleet-grain plain-language render of a P-1 red tick.
- DEFER (Tier-2): the one-click gated repair BUTTON (crosses the P-1 auto-repair changeset seam — lands with P-1 Tier-2) + a full non-coder GUI operator panel (frontend_pair, lands after the CLI/render grain proves out).

## Integration
- crates/orchestrator: the CeremonyProfile type + its journaled selection + the gate-mode plumbing into K-5 + the K-4 tax measurement at the toy tier + the one-line receipt formatter.
- CLI: `elai profile set <tier>` + the fleet render reads through `elai fleet list` (P-1) is the Tier-1 production caller.
- frontend_pair (Tier-2): a fleet operator panel in dashboard-v2 renders the N-7 plain-language cards + the repair button.

## MOVES (wargame)
M1 add the CeremonyProfile type + `elai profile set <tier>` that journals the choice, and thread the tier into K-5's gate-mode selection so the toy tier runs gates OBSERVE-ONLY (run + journal, never block).
EXPECTED_OBSERVATION: setting the toy tier journals a `ceremony_profile_set` row; a subsequent gated action at the toy tier runs the gate, journals its verdict, and does NOT block on a WARN; the same action at the critical tier blocks.
ON_FAIL: the toy tier SKIPS the gate instead of observing it -> evidence is silently dropped (R1 violation) -> make observe-only run+journal+warn, never run-vs-not; assert the journal row exists at the toy tier.
M2 collapse the receipt to one line at the toy/personal tier, the one line POINTING at the full journal entry (never replacing it).
EXPECTED_OBSERVATION: a toy-tier action emits a one-line receipt that links to the full journal row; the full evidence is retrievable from the link.
ON_FAIL: the one-line receipt has no path back to the full evidence -> the collapse became a deletion -> make the one-liner an index into the journal, add the round-trip test.
M3 measure the composed gate tax at the toy tier via K-4 and render a P-1 red tick as one plain-language N-7 sentence.
EXPECTED_OBSERVATION: `elai profile tax --tier toy` reports the measured composed gate tax (must be near-zero for A-3); a P-1 red tick renders as one sentence ("your wardrobe tool broke: its weather dependency is stale").
ON_FAIL: the toy-tier tax is NOT near-zero -> GAP-5 fails (ceremony kills adoption, A-3) -> escalate the specific expensive gate; do not ship a toy tier with a high measured tax.

## FORK
FORK: if K-5's gate-mode is hard-coded per-gate rather than selectable by a caller-supplied tier -> ship Tier-1 by adding a tier parameter to the K-5 selection seam (minimal) rather than restructuring every gate; if that seam does not exist, route the observe-only wiring to the specific gates the toy tier touches (the P-1 fleet ticks) and record the K-5-tier-parameter seam in the decision doc. Do NOT fork the gate engine.

## RECON_NEEDED
RECON_NEEDED: K-5's gate-mode selection API (is strictness a per-gate constant or a caller-supplied tier?) -> settle via `grep -rn "K-5\|risk.tier\|gate.mode\|observe" .elai_cc/crates` + codegraph_explore on the gate-mode selector.
RECON_NEEDED: K-4's composed-gate-tax measurement entrypoint (can it be scoped to a tier?) -> settle via `grep -rn "K-4\|gate.tax\|composed\|ceiling" .elai_cc/crates`.
RECON_NEEDED: N-7's render-target extensibility (can a new fleet-grain plain-language target be added?) -> settle via reading N-7's shipped human-render verdict field + render targets.

## ABORT
ABORT: if a toy-tier profile can only be made near-zero-tax by SKIPPING a gate (not merely running it observe-only) -> stop and flag; W-2/R1 posture forbids silently skipping evidence. A high-tax gate that cannot run observe-only cheaply is a gate-cost problem to file, never a reason to drop the gate at the toy tier.

## WIRING_CONTRACT
produces_symbols:
  - the CeremonyProfile type (declared, journaled) + `elai profile set|tax` CLI subcommand
  - the toy-tier observe-only gate-mode wiring into K-5 + the one-line receipt collapse (indexing the full journal)
  - the N-7 fleet-grain plain-language render + the journaled `ceremony_profile_set` SSB row
consumes_symbols:
  - K-5 risk-tiered gating, K-4 composed gate-tax measurement, N-7 human-render, P-1 fleet red tick, GD-4 observe-first, W-2 kill-switch posture
reachable_from:
  - the `elai profile set` / `elai profile tax` CLI subcommand on a live run  [PRODUCTION CLI]
  - a gated action at the toy tier on a live run runs observe-only + journals  [PRODUCTION gate path]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO tier that SKIPS a gate silently (observe-only runs + journals)
  - NO one-line receipt that replaces (rather than indexes) the full journal
  - NO second gate engine (ride K-5/K-4)
  - NO loosening that isn't a declared journaled profile choice
observable_effect:
  trigger: "set the toy tier, run a gated fleet action, then run the same action at the critical tier"
  expect_ssb: "a ceremony_profile_set row on the tier choice; the gate verdict journaled at BOTH tiers; the toy-tier action does NOT block on WARN, the critical-tier action DOES"
  expect_artifact: "a one-line toy-tier receipt linking to the full journal row; `elai profile tax --tier toy` near-zero; NEGATIVE: a toy tier that drops the journal row, or a one-liner with no path to full evidence, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai profile` subcommand dispatch sets the journaled CeremonyProfile AND the K-5 gate-mode selection seam reads the active tier each gated action — the executor names the exact selector fn at RECON and wires both same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-5_operator_surface_graduated_ceremony.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a gated action ran OBSERVE-ONLY at the toy tier (verdict journaled off-agent, no block) and ENFORCE at the critical tier (blocked), with the one-line receipt resolving to the full journal row and a measured near-zero toy-tier tax.

INTEGRATION_SMOKE: an independent agent sets the toy tier, drives a P-1 red-tick fleet action (gate runs observe-only + journals + one-line renders), escalates the same profile to critical (the same action now blocks), and asserts the journal row exists at BOTH tiers + the measured tax delta between tiers — a multi-tier run firing the observe-vs-enforce + collapse paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "hide a real failure from a non-coder by collapsing the receipt to a green one-liner": FAILS because observe-only still RUNS + JOURNALS the gate and the one-liner INDEXES that journal; the failure is one click away, never deleted. Attack that SUCCEEDED and its patch — "a user drops to toy tier to dodge a blocking security gate on a real trading tool": patched by P-7's asymmetry (a tier that loosens gates is a declared journaled choice and the risk-tier of a tool is a property of the tool, not user-selectable downward past its declared risk) — the toy tier applies to toy-risk tools, not to a critical tool relabeled.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-5 + T1 mapping + A-3 (ceremony-tax-kills-adoption falsifier; CodeGate died of FP friction; GD-4 exists because of it) + A-4 (adaptation-as-sycophancy, patched by P-7 asymmetry).
- reddit r/ClaudeAI thread archive — Hapiro "absolutely 0 understanding of coding".
- feature-map: K-5 risk-tiered gating, K-4 composed gate-tax ceiling, N-7 human-render, J-32/J-35 front-door, L-14 one-verb build, GD-4 observe-first, W-2 kill-switch.

## COMPOSES-WITH
K-5/K-4 (EXTEND with a tier profile, do NOT re-author), N-7 (new fleet-grain target), P-1 fleet red tick, GD-4/W-2. Re-implements NONE (GD-1). P-1 supplies the fleet diagnosis P-5 renders; P-7 owns the asymmetry rule that keeps the toy tier from becoming a gate-dodge.

End of P-5.
