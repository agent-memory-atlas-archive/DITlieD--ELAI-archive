# P-23: journaled co-browse lane — the general-purpose browser surface, with receipts

STATUS: PLANNED (WAVE CU-4; PROOF_SPINE: none (product), owner-commissioned 2026-07-11 (unit explicitly requested in the co-browse gap session off eval ENTRY 14); execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever co-browse-with-receipts | grounding STRONG on the comparator (ClaudeDevs post 2075635283211772279 fetched verbatim via syndication CDN 2026-07-11 + 9to5mac coverage + Tauri multiwebview status searched), MODERATE on ELAI anchors (PLAN-asserted, executor re-greps W-7) | nature EXTENSION (H-15 + E-14) | effort M | WAVE CU-4 | depends H-13,H-15,E-14,P-13 | PROOF_SPINE none (product)
- Premise: Claude Code desktop shipped an in-app browser (2026-07-10): sandboxed clean profile, model reads/clicks any site, session persistence a user choice — and ZERO evidence discipline (no journaled action stream, no replay artifact, no gate between the model's claim and the verdict). eval ENTRY 14 called this ELAI's ONE design-level hole on the axis: no surface where user+model interactively browse an ARBITRARY site together (E-14 is test-scoped, canvas RESEARCH mode is research-scoped, P-12 is a last-resort operator lane). This unit closes it the ELAI way: a co-browse surface where an unjournaled input path is structurally impossible.

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 14. The comparator's trust story is isolation only (clean profile + sandbox) — the same story all six ENTRY-12 repos tell. ELAI's canvas already specs the harder halves: a journaled model-driven op stream (H-13), a taint-gated deixis handoff (H-15 H15-2), provenance-tagged pointing-by-id (P-13), and a crystallize-to-replay lane (P-14). What is missing is only the SURFACE: an embedded browse pane and the one-proxy input discipline. The inversion: they ship the pane without receipts; we ship the receipts and add the pane.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- E-14 Browser E2E (E-SUMMARY_INDEX.md:134): AXTree-first Playwright substrate + axe-core + VLM Tier-2 advisory. The engine this lane REUSES — never a second browser substrate.
- H-13/H-15 (H-SUMMARY_INDEX.md:221,257): canvas op transport + dispatcher; spawn_view reply channel + CanvasUiAction::Select with G-1 taint check at the handoff. The browse node is an H-15 view_type.
- P-13 perception provenance (P-plan): the model points by integer id off a source-tagged element list; raw coordinates only as fallback. P-12 frame_bind.rs: coordinate actions bound to the sha-hashed delivered frame.
- P-14 explore-then-crystallize: a co-browse exploration can mint a deterministic replay script, RRA-bound (compose, restate none).
- G-1 taint / G-2 RRA; P-6 life-data privacy lattice (owned bytes flowing OUT); P-10 credential vault (profile custody); J front-door router (the "pull up X" intent lane).

## Steal/build (tiered)
- Tier-1 READ lane (cheap, the "pull up docs/designs" case): a canvas browse node rendering a page EXTRACTION — E-14 substrate nav + AXTree + screenshot, read-only, content taint-tagged web at ingest, deixis via H-15. No interaction, minimal ceremony (P-5 graduated-ceremony doctrine).
- Tier-2 CO-BROWSE lane: an orchestrator-owned Playwright/CDP browser; CDP screencast frames stream into the browse node — THE PANE IS DUMB GLASS. One InputProxy is the only holder of the engine's input session: every nav/click/keystroke — user (forwarded from the pane) or model (P-13 id-pointing; coordinate fallback under P-12 frame-binding) — lands an actor-tagged cobrowse_input SSB row BEFORE dispatch. Journal completeness is structural, not policy: there is no second input path to bypass.
- Session persistence, typed: default = ephemeral profile destroyed on close; `--persist <name>` = a named profile whose cookie/state custody sits in the P-10 credential vault; the choice is a field on the cobrowse_session row. (The comparator's one-bit choice, typed + gated.)
- Egress gating: user-typed text bound for a web form passes the P-6 privacy lattice + a domain-first-seen warning row before dispatch — the dimension the comparator lacks entirely.
- Authority boundary: everything read in the pane is a G-1 taint source; browse content can never authorize a mutating lane (restate none — the existing taint gate is the enforcement).
- EXCLUDED as the mechanism: a native embedded webview as the interactive surface — Tauri multiwebview is behind the unstable cargo flag with open positioning bugs (#10131/#10420, searched 2026-07-11), it would put remote DOM inside the trusted shell, and native input events would constitute an unjournaled path. It survives only as the FORK's future option, InputProxy still interposed.

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 14 + ENTRY 12 synthesis)
- docs/thefinalglance/E-plan/E-14_*.md (the engine seam), docs/thefinalglance/H-plan/H-15 section (spawn_view + deixis), P-plan/P-12/P-13/P-14 (frame-binding, pointing, crystallize)
- .claude/rules/wiring-enforcement.md (R10), .claude/rules/production-caller-gate.md (D-030)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/cobrowse/mod.rs (CoBrowseLane: session mgr + screencast pump + profile custody hook)
  - crates/orchestrator/src/cobrowse/input_proxy.rs (InputProxy: journal-then-dispatch; owns the engine input handle PRIVATELY — the only input path, enforced at the type level)
consumes_symbols:
  - crates/orchestrator/src/browser_e2e/ engine (E-14; if E-14 is unexecuted at edit, THIS unit vendors the substrate and E-14 consumes it — threaded contract, not a second engine)
  - crates/orchestrator/src/perception/mod.rs::PerceptionScaffold (P-13 element ids), crates/orchestrator/src/operator/frame_bind.rs::FrameBinding (P-12; coordinate fallback)
  - crates/orchestrator/src/taint.rs::TaintSource (G-1), crates/orchestrator/src/rra.rs::ActionStream (G-2)
reachable_from:
  - crates/cli/src/main.rs (`elai browse <url>`) [PRODUCTION CLI]
  - the front-door router lane table ("pull up / open <url>" resolves to the browse lane) [PRODUCTION router]
  - the H-13 canvas dispatcher (browse node view_type + forwarded pane input) [PRODUCTION frontend]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO input path that bypasses InputProxy (structural: the CDP input session handle is private to the proxy)
  - NO model coordinate action without a live P-12 frame binding; NO persistent profile outside P-10 vault custody
  - NO remote DOM inside the trusted dashboard webview; NO .unwrap()/.expect() on the lane; blake3 for frame hashes
frontend_pair:
  the canvas browse node: screencast render (Tier-2) / extraction view (Tier-1) + an actor-colored input trail so the user SEES the journal as it lands.
observable_effect:
  trigger: "elai browse --fixture tests/fixtures/cobrowse/docs_page.json"
  expect_ssb: "cobrowse_session"
  expect_artifact: "every dispatched input has a PRIOR actor-tagged cobrowse_input row (journal-then-dispatch ordering checked off-agent against the engine's own CDP log); NEGATIVE: an input reaching the page with no prior row, a model coordinate action on a stale frame hash, or an ephemeral session leaving profile bytes on disk, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: opening a URL lands one cobrowse_session row {url, profile: ephemeral|vault:<name>}; each user click forwarded from the pane and each model op lands exactly one cobrowse_input row {actor, target: element-id|coord+frame_hash} before the engine dispatches it; the browse node renders the live screencast; closing an ephemeral session leaves zero profile bytes.
ON_FAIL: an input reaches the page without a row (CAUSE: a second input path — a native webview event, a direct CDP call from outside the proxy — exists). COUNTER-MOVE: move the engine input handle into InputProxy's private field so outside dispatch cannot compile; add the negative arm that fires a raw CDP input and asserts the journal-completeness check FAILS the run (fail-closed proven, not assumed).
FORK: if co-browse input-to-paint p95 exceeds 250ms on the local loop (measured, not vibes), evaluate the native Tauri multiwebview pane — ONLY once multiwebview has left the unstable flag AND with InputProxy still interposed; until both hold, the screencast stays. If the pinned engine lacks CDP screencast, fall back to a timed captureScreenshot loop (degraded FPS, identical receipts).
RECON_NEEDED: confirm the pinned Playwright version exposes raw CDPSession + Page.startScreencast (settling check: WebSearch + a source read of the pinned release at edit); settle Rust-native driver vs the bun/Playwright capture sidecar (visual-ui-loop precedent) with an anti-slopsquat registry check on any crate named; re-check Tauri multiwebview flag status at the FORK tripline; confirm the front-door lane-table seam name in the shipped router (grep at edit).
ABORT: any discovered input path that does not pass InputProxy (P0 — the property IS the unit); a persistent profile written outside vault custody; profile isolation failure (a session seeing personal browser state — clean-profile parity with the comparator is table stakes, not a feature).
VERIFY: the load-bearing proof is the off-agent cobrowse_session + cobrowse_input rows and the ordering check driven by the real `elai browse` entrypoint (never a unit test calling the lane). Run at execution:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md   # execution done-gate
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/cobrowse/mod.rs,crates/orchestrator/src/cobrowse/input_proxy.rs   # no DARK symbol
```
REDTEAM: attack that FAILED — a hostile page renders a fake ELAI approval dialog inside the pane (UI redress): defeated structurally, harness chrome never renders inside the screencast and a pane click can only ever become a cobrowse_input row, never a harness command; approvals live in native dashboard chrome outside the node. Attack that FAILED — page text instructs the model to click a destructive control: browse content is G-1-tainted and untrusted-content->destructive-sink is already blocked (restate none). Attack that SUCCEEDED unpatched — the user types a secret into a phishing form (the proxy journals it but journaling is not protection): PATCH — outbound typed text passes the P-6 privacy lattice + a domain-first-seen warning row BEFORE dispatch; egress is gated, not merely recorded. Deep red-team DEFERRED to execution pre-flight (legacy refine-before-execute path, as P-9..P-22).

## INTEGRATION_SMOKE
After P-23 + P-13 land: an independent agent drives `elai browse` end-to-end on a HARD input — a real docs site PLUS a hostile fixture page (injection strings + a fake-dialog overlay), a multi-step interleaved user+model session long enough to trigger a mid-run compaction — and asserts (a) every input has its prior actor-tagged row, (b) the injected instruction never reaches a mutating lane, (c) the ephemeral profile leaves zero bytes, (d) a P-14 crystallize handoff mints a replay script RRA-bound to this session. A toy single-nav happy path does NOT satisfy R10.

## Grounding / citations
- Comparator: x.com/ClaudeDevs/status/2075635283211772279 (verbatim tweet JSON via cdn.syndication.twimg.com, fetched 2026-07-11); 9to5mac.com 2026-07-10 (clean-profile / persistence detail); workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 14 (the design comparison this unit closes).
- Tauri multiwebview unstable + open bugs: v2.tauri.app release notes + tauri-apps/tauri issues #10131/#10420 (WebSearch 2026-07-11) — the grounded reason the native pane is a FORK, not the mechanism.
- ELAI anchors (PLAN-asserted, executor re-greps W-7): E-14 (E-SUMMARY_INDEX.md:134), H-13/H-15 (H-SUMMARY_INDEX.md:221,257-275), P-12/P-13/P-14, G-1/G-2, P-6, P-10, O-2 per their files.
- CLAUDE.md hard rules invoked: blake3 not sha2; no .unwrap on prod; anti-slopsquatting on any browser-driver crate; bun-not-node if the capture sidecar is JS.

## PRODUCTION_HOOK
crates/cli/src/main.rs (`elai browse <url>`), the front-door router lane table (a browse lane the "pull up X" intent resolves to), and the H-13 canvas dispatcher (browse node view_type + pane-input forwarding) — the new CoBrowseLane/InputProxy fns are wired into ALL THREE in this changeset (vertical slice; not a later step).

## Next
Closes the ENTRY 14 gap — the one design-level hole the Claude Code in-app browser exposed. Pairs with P-13 (pointing), P-14 (a co-browse exploration crystallizes to replay), P-16 (the catalog provisions what the pane would otherwise be used to fetch). The pitch line it buys: the only co-browse surface where the browsing session itself is journaled, replayable, taint-bounded evidence.

End of P-23.
