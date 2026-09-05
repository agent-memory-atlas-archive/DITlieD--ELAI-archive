# N-7: human render as a REQUIRED verdict field — one render substrate, every verdict producer declares its render

- Track: TIER 2 (HIGH) | lever: a-non-author-can-see-the-value | finding: W-7 | grounding: MODERATE | nature: BUILD | effort: M | WAVE 5

- Premise: the engine spec is ~200 units deep; the surfaces where a human actually SEES the trust value are few and late. Evidence is machine-shaped by default (RRA bundles, SSB rows, hashes); only K-11 converts it to human-viewable form and only for PRs. The reasoning receipts, hallucination verdicts, and measurement numbers each produce human-decision-relevant artifacts with no render but a drawer. The differentiators only sell if a non-author can see them.

## Steal/build
Make "human render" a REQUIRED field of every gate/verdict family the way +SSB-row is required today: every verdict class declares its K-11-style render target — PR comment / chat ACK line / canvas badge / brief row / M-3 org rollup. Build ONE render substrate (a render(verdict, target) dispatcher) with many verdict producers plugging in, instead of K-11 being a one-off for G-5. The N-2 authoring gate then makes render_field a REQUIRED contract field (a new verdict class with no declared render fails the corpus check).
Verdict families that gain a render: J-83 receipts, L-7 hallucination verdicts, N-1 measurement numbers, N-5 RouteDecision, N-6 provenance, N-8 governance-coverage.
EXCLUDED: re-authoring K-11 per surface (make it a substrate); a new dashboard framework (compose the existing Svelte dashboard-v2 + K-7 brief + M-3 rollup); inventing a verdict (N-7 renders existing verdicts).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- K-11 PR-evidence render, K-7 buyer brief, G-5 reviewer cockpit, M-3 fleet rollup, the canvas altitudes exist (feature-map render group); KD-2 "meet the buyer in the PR" + K-20 wedge concede distribution is the gap.
- verdicts are emitted as SSB rows + RRA bundles today; only K-11 humanizes them, PR-only.
- dashboard-v2 (Svelte 5 + Tauri v2) is the frontend; the drawers are the current render surface.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the render(verdict, target) substrate + render_field as a required contract field (wired to N-2) + rendering 2-3 verdict families (J-83 receipts + L-7 + N-1 numbers) at their declared targets.
- DEFER (Tier-2): the M-3 org-rollup render + full canvas-badge coverage (land as the org/canvas units mature).

## Integration
- crates/orchestrator verdict emission + the render substrate; dashboard-v2 + K-11/K-7 as render targets.
- frontend_pair: MANDATORY — this IS the frontend unit; cross_lang_diff over the Tauri command + Svelte render targets.

## WIRING_CONTRACT
produces_symbols:
  - the render(verdict, target) substrate (dispatcher + the target adapters: PR/chat/canvas/brief/rollup)
  - render_field as a required WIRING_CONTRACT field (enforced by N-2)
consumes_symbols:
  - K-11 PR render (the template), K-7 brief, G-5 cockpit, M-3 rollup, dashboard-v2 drawers
  - the verdict producers (J-83/L-7/N-1/N-5/N-6/N-8)
reachable_from:
  - a live verdict-emitting gate -> render at its declared target  [PRODUCTION verdict seam]
  - the dashboard/Tauri render path  [PRODUCTION frontend]
stub_ban:
  - NO per-surface re-authoring of K-11
  - NO verdict class shipped with no declared render (N-2 fails it)
  - NO new dashboard framework / new dep
frontend_pair: dashboard-v2 render targets + K-11 PR comment; cross_lang_diff PASS. MANDATORY.
observable_effect:
  trigger: "a live verdict-emitting gate (e.g. an L-7 hallucination verdict on a real task)"
  expect_ssb: "+verdict_rendered(PrComment|ChatAck|CanvasBadge|BriefRow|OrgRollup)"
  expect_artifact: "a human-readable render lands at the declared target (visible in the dashboard-smoker); NEGATIVE: a new verdict class with no declared render fails the N-2 authoring check"

## Grounding
- interlace doc W-7 (value-render layer thin vs engine depth), TIER-2 item 7; KD-2/K-20 (distribution gap).
- dashboardRules.md + visual-ui-loop.md (the Svelte render doctrine); K-11 as the existing render template.

## COMPOSES-WITH
K-11, K-7, G-5, M-3, dashboard-v2, the verdict producers. Re-implements NONE (GD-1). The engine's depth only sells if a non-author can see it; N-7 makes render a first-class requirement, not a one-off.

End of N-7.
