# P-24: mobile design/test lane — the deferred H-14 smartphone leg, three rungs on the P computer-use spine

STATUS: PLANNED (WAVE CU-5; PROOF_SPINE: none (product), owner-commissioned 2026-07-11 (requested in the co-browse gap session after the smartphone-coverage question); Tier-3's crystallize rung transitively feeds PS-5 via P-14 (declared per PSR-1 tier 3); execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever mobile-as-a-dimension-not-a-subsystem | grounding STRONG on the external legs (Expo web production-grade on SDK 54+ via Metro/react-native-web, docs.expo.dev/workflow/web + 2026 guides, searched 2026-07-11; Maestro mobile-dev-inc single-binary YAML flows on Android emulator/iOS simulator, github.com/mobile-dev-inc/maestro + maestro.dev, searched 2026-07-11), MODERATE on ELAI anchors (PLAN-asserted, executor re-greps W-7) | nature EXTENSION (H-14 + P-12/P-13/P-14 + E-14) | effort L | WAVE CU-5 | depends H-14(B0+tagging),P-12,P-13,P-14,P-16 | PROOF_SPINE none (product)
- Premise: H-14 is explicitly web-first — "React Native / Expo deferred to a later separately-scoped slice (nothing here promises it)" (H-14_design_mode_web_first.md:7), and H-14:184 records the sting: the current gig's locked stack IS RN. Since that deferral, the P computer-use wave built exactly the machinery the native leg was waiting for (P-12 sandboxed operator lane, P-13 provenance-tagged perception, P-14 explore-then-crystallize, P-16 gated install catalog). This unit is the named later slice: mobile as a typed DIMENSION threaded through existing lanes, never a parallel mobile subsystem.

## Context — why this matters
Corpus grep (2026-07-11): zero smartphone/mobile/Expo/RN/emulator coverage in any letter beyond H-14's deferral, 4D/4E's ConsumerMobileApp product type, and B-42's store-guideline artifacts. No device-frame preview, no mobile-viewport e2e, no emulator lane, no RN click-to-edit. Meanwhile the pipeline factory can PLAN a ConsumerMobileApp it can neither preview, click-edit, nor e2e-test — a plan-to-proof hole on a product type ELAI already claims. The three rungs close it cheapest-first, each with the receipts doctrine intact.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- H-14 (H-plan file): dev-preview proxy (B0, frameable origin), build-time opaque data-oid tagging with SERVER-SIDE id->{file,range,scope}, two-tier recast writeback. The RN mechanism is pre-named in its doctrine: Fiber + getInspectorDataForViewAtPoint, native render not iframe (H-14:184).
- E-14 (E-SUMMARY_INDEX.md:134): AXTree-first Playwright journeys + SSIM diff — gains a device matrix, not a sibling.
- P-12 operator lane: disposable sandbox + FrameBinding (hash-bound coordinate actions) + arm/lease — an emulator is just another sandboxed GUI. P-13: pointing by integer id off a source-tagged element list; counts never from a VLM (J-1). P-14: exploration mints a deterministic replayable script, RRA-bound. P-16: the admission-gated install catalog (SDK images, emulator binaries, Maestro). P-23: the browse node + InputProxy (a device-frame is a render variant of the same node).
- 4I media validator (render-capture-evaluate, Tier-1 veto) — the screenshot-judging doctrine reused per rung; 4D/4E ConsumerMobileApp product type + B-42 ios/android store-guideline artifacts (the planning-side context this lane finally makes testable).

## Steal/build (three rungs, cheapest first; every rung labels its render source — a lower rung NEVER discharges a higher rung's gate)
- RUNG 1 DEVICE-FRAME VIEWPORT (web-on-phone-glass): a typed DeviceProfile catalog as DATA rows {name, width, height, dpr, ua, touch} (named constants, no magic metrics in code); a device_frame render variant on the H-14 preview node and the P-23 browse node (CDP Emulation.setDeviceMetricsOverride — the repo's own capture scripts already pin deviceScaleFactor, precedent); E-14 journeys accept a device MATRIX and the SSIM goldens become per-profile. Closes "does my web app work on a phone" for near-config cost.
- RUNG 2 EXPO-WEB DESIGN MODE (RN apps on the H-14 surface): an Expo dev server (Metro web, production-grade SDK 54+) is a frameable origin, so H-14's B0 proxy + data-oid tagging + recast writeback extend to RNW output unchanged in mechanism; click-to-edit resolves RN component AST ranges exactly as web. Render source labeled rnw_web — an RNW pass is a WEB pass on RN source, never claimed as a native pass (react-native-web fidelity gaps are the reason the label exists).
- RUNG 3 NATIVE EMULATOR LANE (the real deferral): an Android emulator (and iOS simulator where a macOS host exists) as a P-12 sandbox backend — booted from a PINNED AVD/sim snapshot whose image hash lands in the RRA hardware pin; perception = the platform accessibility tree (uiautomator dump / XCUITest a11y) as a NEW P-13 source tag mobile_axtree (pixel/OCR tiers fire only when it is empty, J-1 doctrine unchanged); actions frame-bound per P-12; an agentic exploration run crystallizes via P-14 into a Maestro YAML flow, RRA-bound to the minting run, replay = the zero-reasoning mode; P-16 catalog provisions emulator images/SDKs/the Maestro binary (admission-gated, never ad-hoc install). RN click-to-edit-on-native (Fiber getInspectorDataForViewAtPoint) stays a REGISTERED FOLLOW-UP inside this rung's FORK, not a promise.
- EXCLUDED: a parallel "mobile subsystem" (every rung is a dimension on an existing lane); VLM-read screenshots as the element source (P-13/J-1); real physical devices (emulator/simulator only — a physical device is neither disposable nor snapshot-pinnable, ABORT class).

## Reads (mandatory before coding)
- docs/thefinalglance/H-plan/H-14_design_mode_web_first.md (B0 proxy, tagging, writeback, the RN caveat at :184)
- P-plan/P-12/P-13/P-14/P-16/P-23 (the machinery each rung composes), docs/thefinalglance/E-plan/E-14_*.md
- .claude/rules/wiring-enforcement.md (R10), .claude/rules/production-caller-gate.md (D-030)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/mobile/device_profile.rs (DeviceProfile catalog rows + resolver; DATA not code)
  - crates/orchestrator/src/mobile/emulator_backend.rs (P-12 SandboxBackend impl: pinned-snapshot AVD/sim boot + mobile_axtree feed + Maestro-flow crystallize target)
consumes_symbols:
  - crates/orchestrator/src/browser_e2e/ engine (E-14; device matrix = a parameter, never a second runner)
  - crates/orchestrator/src/operator/frame_bind.rs::FrameBinding + crates/orchestrator/src/perception/mod.rs::PerceptionScaffold (P-12/P-13; do NOT re-implement)
  - the P-14 crystallizer (Maestro flow = a new script target enum arm) + the P-16 install catalog (emulator/SDK/Maestro provisioning)
reachable_from:
  - crates/cli/src/main.rs (`elai e2e run --device <profile>`; `elai operator run --backend emulator`) [PRODUCTION CLI]
  - crates/orchestrator/src/task_runner.rs (device_profile field on browse/e2e/operator TaskSpec stages) [PRODUCTION task_runner]
  - the H-13 canvas dispatcher (device_frame render variant on preview/browse nodes) [PRODUCTION frontend]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO rung-label laundering: every result row carries render_source (web_viewport | rnw_web | native_emulator); a lower-rung pass discharging a higher-rung gate is a gate defect
  - NO element counts from a VLM on mobile trees (J-1); NO unpinned emulator image (snapshot hash in the RRA or ABORT); NO physical devices; NO hardcoded device metrics (catalog DATA only)
  - NO .unwrap()/.expect() on the lane; blake3 for snapshot/frame hashes; Maestro binary only via the P-16 catalog (anti-slopsquat at edit)
frontend_pair:
  the device_frame node variant (phone-bezel render of H-14 preview / P-23 browse at a selected DeviceProfile) + per-profile pass/fail chips on the e2e journey node; iOS-unavailable renders as an HONEST-GAP disconnected affordance, never a silent skip.
observable_effect:
  trigger: "elai e2e run --device pixel8 --fixture tests/fixtures/mobile/journey_web.json"
  expect_ssb: "emulator_session"
  expect_artifact: "the journey's axtree_snapshot rows carry device_profile=pixel8 + render_source labels; a Rung-3 run boots from a pinned snapshot whose hash lands in the RRA hardware pin and crystallizes a Maestro flow RRA-bound to the run; NEGATIVE: a rnw_web pass presented against a native_emulator gate, an emulator boot from an unpinned image, or a mobile count sourced from a VLM, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: Rung 1 — the same E-14 journey run under --device pixel8 emits per-profile rows + per-profile SSIM goldens; the canvas shows the device-frame render. Rung 2 — an Expo fixture app's pages render as H-14 preview nodes through the same proxy and a click resolves to the RN component's AST range. Rung 3 — one emulator_session row {snapshot_hash, backend}, frame-bound operator_action rows, and a crystallized Maestro flow bound to the exploration run's RRA.
ON_FAIL: the crystallized Maestro flow passes on replay while the agentic run's assertions actually failed (CAUSE: the flow encoded taps but not the off-agent assertions — Maestro's assertVisible is the flow's own claim, not ELAI's). COUNTER-MOVE: replay verdicts come from the SAME off-agent evidence as the minting run (SSB rows + screenshot hash compare per 4I), Maestro is actuation-only; add the negative arm where a mutated fixture makes replay actuate green while the evidence leg FAILS the run.
FORK: no macOS host at execution -> iOS Tier-3 is marked unavailable and rendered as the honest-gap affordance (Android-only lane; NEVER substitute a rnw_web run for the missing iOS leg). Expo fixture's a11y tree arrives empty from uiautomator on RN (known RN accessibility-props dependence) -> P-13's tier ladder fires (OCR/pixel) AND the run records perception_degraded — it does not silently pretend axtree_exact. Maestro binary fails the P-16 admission gate -> fall back to driving via the P-12 actuator with a plain journaled step list as the crystallize format (uglier, same receipts); do NOT install around the catalog.
RECON_NEEDED: verify Maestro's license + latest release + binary checksum channel at edit via the registry/repo (anti-slopsquat; the P-16 catalog row needs license_tag); verify the pinned Expo SDK's web output still tags data-oid through Metro builds (settling check: run H-14's tagging step on an Expo web fixture and grep the emitted DOM); verify uiautomator dump fidelity on an RN fixture (settling check: dump the tree on the fixture app, count labeled nodes vs rendered controls); confirm the CDP device-metrics call sites in the repo capture scripts as the Rung-1 precedent (grep deviceScaleFactor under dashboard-v2/screenshots + .claude/tools/visual).
ABORT: any path that would drive a PHYSICAL device (not disposable, not snapshot-pinnable — P0); an emulator that can reach the host filesystem/network outside its sandbox profile; a rung-label missing on any result row (the honesty mechanism IS the unit); the H-14 B0 proxy still unshipped at execution (this unit extends it, it cannot front-run it — STOP and flag, do not re-implement a proxy).
VERIFY: the load-bearing proof is off-agent per rung: per-profile axtree_snapshot/SSIM rows (Rung 1), the click->AST-range resolution on a real Expo fixture (Rung 2), the emulator_session + RRA-bound Maestro flow + evidence-leg replay verdict (Rung 3) — all driven by the real `elai e2e run` / `elai operator run` entrypoints, never a unit test calling the backend. Run at execution:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md   # execution done-gate
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/mobile/device_profile.rs,crates/orchestrator/src/mobile/emulator_backend.rs   # no DARK symbol
```
REDTEAM: attack that FAILED — claim mobile coverage off the cheap rung (run rnw_web, report "mobile tested"): defeated structurally, render_source is a typed field on every row and a Tier-2 pass cannot discharge a Tier-3 gate (the result manifest names the rung; the NEGATIVE arm tests the laundering directly). Attack that FAILED — a flaky emulator silently re-boots a different image until green: the snapshot hash is in the RRA hardware pin and a hash mismatch is an ABORT, not a retry. Attack that SUCCEEDED unpatched — the crystallized Maestro flow becomes the verdict authority on replay (its own assertVisible claims): PATCH — Maestro is actuation-only; replay verdicts come from the same off-agent SSB/screenshot evidence leg as the minting run (folded into ON_FAIL's counter-move + the negative arm). Deep red-team DEFERRED to execution pre-flight (legacy refine-before-execute path, as P-9..P-23).

## INTEGRATION_SMOKE
After P-24 lands (with P-12/P-13/P-14 present): an independent agent runs the SAME journey across all three rungs on a HARD input — a real Expo fixture app with an intentionally broken phone-only layout + one control lacking accessibility props — and asserts (a) Rung 1 catches the layout break per-profile while desktop passes, (b) Rung 2 click-to-edit resolves the broken component's AST range, (c) Rung 3 boots pinned, degrades perception HONESTLY on the unlabeled control (perception_degraded, no VLM count), crystallizes a Maestro flow, and the replay's evidence leg reproduces the minting verdict, (d) every row carries the correct render_source. A single-rung happy path does NOT satisfy R10.

## Grounding / citations
- H-14 deferral + pre-named RN mechanism: docs/thefinalglance/H-plan/H-14_design_mode_web_first.md:1,7,184 (read 2026-07-11).
- Expo web production-grade SDK 54+ (Metro, react-native-web; RNW powers x.com web): docs.expo.dev/workflow/web + reactnativerelay.com 2026 guide + necolas.github.io/react-native-web (searched 2026-07-11). RNW fidelity gaps = the reason for the rnw_web label.
- Maestro (mobile-dev-inc): single-binary, declarative YAML flows, Android emulator/iOS simulator/RN/web, deterministic-leaning interpreted engine: github.com/mobile-dev-inc/maestro + maestro.dev + browserstack.com/guide/maestro-testing (searched 2026-07-11); license/checksum verified at edit per anti-slopsquat (RECON).
- CDP device-metrics precedent in-repo: the pinned deviceScaleFactor discipline in the capture harness (visual-ui-loop rule + dashboard-v2/screenshots cdp_*.mjs), grep at edit.
- ELAI anchors (PLAN-asserted, W-7): E-14, P-12/P-13/P-14/P-16/P-23, 4I, 4D/4E ConsumerMobileApp, B-42 store-guideline artifacts per their files/indexes.
- CLAUDE.md hard rules invoked: blake3; no .unwrap on prod; anti-slopsquatting (Maestro, Expo SDK pin); no new deps without confirmation (Maestro enters ONLY through the P-16 catalog).

## PRODUCTION_HOOK
crates/cli/src/main.rs (`elai e2e run --device`, `elai operator run --backend emulator`), crates/orchestrator/src/task_runner.rs (device_profile on browse/e2e/operator stages), and the H-13 canvas dispatcher (device_frame render variant) — DeviceProfile resolver + emulator backend wired into ALL THREE in this changeset (vertical slice; Rung 3's crystallize arm registers in the P-14 target enum same-changeset).

## Next
Closes the H-14 smartphone deferral as three honest rungs on the P computer-use spine. With P-24, the 4D/4E ConsumerMobileApp product type finally has preview (device-frame), click-to-edit (Expo web), and e2e-with-receipts (emulator + Maestro replay) — and the rung labels make "mobile tested" a typed claim instead of a vibe. Native RN click-to-edit (Fiber inspector) remains the registered follow-up inside Rung 3's FORK.

End of P-24.
