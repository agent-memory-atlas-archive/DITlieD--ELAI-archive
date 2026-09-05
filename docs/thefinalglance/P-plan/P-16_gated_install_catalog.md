# P-16: gated install catalog (capability-on-demand, fail-closed)

STATUS: PLANNED (WAVE CU-3; EXTENSION of P-9 + P-10 + L-12 admission; PROOF_SPINE: none (distribution), owner-acked 2026-07-11; execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever capability-on-demand-gated | grounding MODERATE (openclaw ClawHub + open-computer-use tools_config verified; the fail-closed inversion is ELAI's) | nature EXTENSION | effort M | WAVE CU-3 | depends P-9,P-10 | PROOF_SPINE none (distribution)
- Premise: the field norm is install-on-demand (agent-zero ungated apt-get, open-computer-use tools_config catalog, openclaw installPolicy-gated registry). P-9's pack lane is the gated answer but nothing specs the CATALOG that makes capability-on-demand FEEL instant. This adds it, behind the existing admission family. The owner's "workflows that install programs (blender, ps) on demand" idea, done fail-closed.

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 3/4/8 + SYNTHESIS 2c/P7. The sanctioned form for "download Blender/Photoshop" is a P-9 HEADLESS code-pack connector wrapping the app's CLI/API (blender --background --python recipe.py; Unity via MCP) — deterministic, journalable, replayable — NOT GUI computer-use. TEIKOKU-asset-studio convergence (ENTRY 4): "Blender and Unity are downstream EXECUTION STAGES driven headless, never by GUI computer-use"; "provider websites without a supported API are isolated OPERATOR LANES, not load-bearing infrastructure". open-computer-use's tools_config.json (ENTRY 8) is the SEED of the allowlisted install mechanism (named {tool, mirror, alias} rows) — but that repo does not gate; ELAI does, via the L-12/skillspector admission family.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- P-9 connector/stage-PACK format (P-0_INDEX.md SECOND WAVE): a signed, versioned declarative pack (one generic http_call kind parameterized by a DATA ConnectorSpec) + a code-pack lane (B-37). The install target IS a pack — EXTEND.
- P-10 ingress adapters + credential vault (P-0_INDEX.md SECOND WAVE): the auth plane the install/connector surface needs (secrets never journaled/exported).
- L-12 admission / skillspector gate (harness skill-import-policy): the fail-closed local admission the install path rides — reuse, do NOT add a second scanner.
- N-6 provenance lattice: the license_tag + source provenance dimension.
- P-12 desktop operator lane: the GUI-only LAST resort when an app has no headless CLI.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): an install catalog table ({tool, source, mirror, alias, license_tag} rows) + an installPolicy admission check on the L-12 path; `elai connector install <tool>` resolves a catalog row to a P-9 HEADLESS code-pack (deterministic wrapper of the app's CLI/API).
- DEFER (Tier-2): the hidden-until-verified registry release gate (openclaw S17: publish -> scan -> HIDDEN until review passes) composed with M distribution; per-OS mirror selection.
- EXCLUDED as vaporware: agent-zero's ungated apt-get pole; openclaw's advisory regex scanner (V5 — signature-match with no teeth is the pattern our gate doctrine rejects); a GUI-driving install when a headless CLI exists.

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 3/4/8, SYNTHESIS P7); workflow/reasoning/TEIKOKU-ASSET-STUDIO-*.md (the headless-stage convergence)
- docs/thefinalglance/P-plan/P-9_*.md + P-10_*.md (the pack format + vault to extend), .claude/rules/skill-import-policy.md (the admission gate)
- crates/orchestrator/src/ (the connector/pack module + L-12 admission seam)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/connector/install_catalog.rs (InstallCatalog: {tool,source,mirror,alias,license_tag} rows + installPolicy)
  - data/connector/install-catalog.toml (the named catalog data, admission-gated)
consumes_symbols:
  - crates/orchestrator/src/connector/pack.rs::ConnectorPack (P-9; the install target is a headless code-pack — EXTEND)
  - crates/orchestrator/src/admission.rs::SkillspectorGate (L-12; the fail-closed admission — reuse, no second scanner)
reachable_from:
  - crates/cli/src/main.rs (`elai connector install`) [PRODUCTION CLI]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO install that bypasses the L-12 admission family (every catalog admit passes skillspector + the license_tag check)
  - NO GUI-driving install when a headless CLI exists (headless P-9 pack first; P-12 GUI stage is last resort)
  - NO second install scanner (reuse L-12; do NOT port openclaw's advisory regex scanner)
  - NO secret written to the catalog/journal (credentials live in the P-10 vault); NO .unwrap()/.expect() on prod; NO blake2/sha2
frontend_pair:
  the catalog + the admission verdict in the connector surface (Tier-2; W-4).
observable_effect:
  trigger: "elai connector install blender --catalog data/connector/install-catalog.toml"
  expect_ssb: "install_catalog_admit"
  expect_artifact: "one install_catalog_admit row {tool, source, license_tag, verdict} per attempt; a clean-license tool mounts a P-9 headless pack, an installPolicy-flagged tool REFUSES (no install); NEGATIVE: an install that bypasses the L-12 admission family, or a GUI-driving install when a headless CLI exists, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: installing a clean-license catalog tool mounts a P-9 headless pack + lands one install_catalog_admit {verdict=admit} row; installing an installPolicy-flagged tool lands {verdict=refuse} and does NOT install.
ON_FAIL: an install proceeds without passing L-12 admission (CAUSE: the catalog path has its own bypass instead of routing through the skillspector gate). COUNTER-MOVE: route every catalog admit through admission.rs::SkillspectorGate; assert a flagged tool is refused; the negative test (bypass) must FAIL.
FORK: if a requested tool has NO headless CLI/API (GUI-only, e.g. a legacy desktop app), route to a P-12 operator-lane stage (taint-tagged operator lane, never load-bearing) rather than refusing outright OR driving it as infrastructure; record the operator-lane classification.
RECON_NEEDED: confirm the exact P-9 ConnectorPack + L-12 SkillspectorGate symbols to compose, and that blender/unity expose the headless CLI the catalog assumes (blender --background --python is documented; verify at edit). Settling check: WebSearch blender headless CLI + `grep -n "ConnectorPack\|SkillspectorGate" crates/orchestrator/src/`.
ABORT: if a catalog install could ever execute unadmitted code (skip the L-12 gate), STOP and flag — this is the exact ClawHub-with-no-default-gate failure (V5) the whole rule exists to beat.
VERIFY: the load-bearing proof is the off-agent install_catalog_admit row + the refusal of a flagged tool, driven by the real `elai connector install` entrypoint. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-16_gated_install_catalog.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-16_gated_install_catalog.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a slopsquatted/typo catalog entry is defeated by the license_tag + skillspector admission (the same L-12 gate that guards skills; anti-slopsquatting per CLAUDE.md). Attack that would succeed if unpatched: a catalog row that resolves to an arbitrary shell install bypassing the pack lane — PATCH: a catalog row resolves ONLY to a P-9 pack (declarative http_call or a reviewed code-pack), never a raw shell string. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives `elai connector install` for a real headless tool (blender) end-to-end: the catalog resolves it, the L-12 gate admits it, a P-9 headless pack mounts, and a subsequent workflow node drives blender --background; then a flagged/typo tool is refused. Toy stub-catalog does NOT satisfy R10. (Composes with P-9/P-10's own smokes.)

## Grounding / citations
- Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 3/4/8, SYNTHESIS P7; TEIKOKU-ASSET-STUDIO plan (headless-stage convergence).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): P-9, P-10, L-12/skillspector, N-6, P-12 per the indexes cited.
- CLAUDE.md hard rules: anti-slopsquatting on every catalog entry; fail-closed admission (skill-import-policy); no second scanner; blake3; no .unwrap on prod.

## PRODUCTION_HOOK
crates/cli/src/main.rs — `elai connector install` resolves a catalog row and drives it through admission.rs::SkillspectorGate to a P-9 pack mount. The InstallCatalog + installPolicy fns are wired into that CLI path in this changeset.

## Next
Closes SYNTHESIS 2c/P7 (self-extension ergonomics, fail-closed) and gives the owner's "download blender/photoshop on demand" idea a sanctioned home. Pairs with P-12 (the GUI last-resort) and P-9 (the pack the catalog resolves to).

End of P-16.
