# P-3: the persona/mechanism split + hand-this-box-to-a-friend export — untransferability is a factoring failure, not a law of nature

STATUS: PLANNED (WAVE 3; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — the export bundle is a build; the factoring rule is doctrine)

- Track: PERSONAL-INCARNATION | lever: mechanism-vs-persona-factoring | source finding: GAP-3 (answers T5 share-optionally) | grounding: MODERATE (index-derived, PLAN-asserted) | nature: bundle=build, factoring=doctrine | effort: M | WAVE 3 | depends P-1 (manifest) + P-2 (ports)

- Premise: the thread treats untransferability as a law of nature — "[my tool] will immediately break if anyone else looks at or touches it" (heynoswearing). It is actually a FACTORING failure: the weirdness lives interleaved with the mechanism. ELAI already has the correct doctrine in miniature — FS-1 user-extends-via-DATA (L-12/L-13), O-plan A4 two-masters (substrate ships as code, topic packs as data). P-3 generalizes it to whole apps: every built tool is factored MECHANISM (code, tests, gates) vs PERSONA (the owner's rules rows, memory namespaces, thresholds, aesthetic tokens — all DATA). `elai app export` emits a bundle; the recipient's seat re-verifies (M-5 elai-verify), mounts, and RE-PERSONALIZES by regenerating the persona layer against THEIR own memory — the tool survives the handoff because the unshareable part was data all along. This is also the honest answer to Kind_Silver_1921's marketplace wish ("someone needs to make a site to upload their best vibecoded software") and skerit's "now it's actually quite trivial to fix all of these issues".

## Steal/build
An `AppBundle = { mechanism: (code + tests + gates), manifest (P-1 + P-2 ports), rra_receipt (L-14/G-2 lineage), provenance: G-9-redacted }`. The PERSONA layer (rules rows, memory namespaces, thresholds, aesthetic tokens) is EXCLUDED from the bundle by construction — it is regenerated on the recipient's seat. `elai app export <app>` writes the bundle + a receipt; `elai app import <bundle>` runs M-5 elai-verify against the bundle's evidence, refuses on a failed receipt, mounts the mechanism, then regenerates persona against the importer's E-5 memory. The sharing surface is admission-gated: every artifact carries a receipt and passes the same L-12/skillspector admission gates that already exist for skills — a friend-to-friend and marketplace transfer share one gate.
EXCLUDED: exporting the persona layer (defeats the factoring — the persona is regenerated, never shipped); an unreceipted/unredacted bundle (G-9 write-time redaction is mandatory on the provenance leg); a bundle that mounts WITHOUT M-5 re-verification (A-import-must-verify — the recipient re-verifies, never trusts the sender's word); a per-recipient code fork (mechanism is shared verbatim; only persona differs — A-5).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- FS-1 / L-12 / L-13: users extend ELAI via DATA (rules-as-data, skills, topic packs) — the persona layer is exactly this shape, one grain up.
- M-5 elai-verify is the OSS verifier; M-6 is the evidence format — import re-verification rides M-5.
- L-12 + skillspector (/skill-audit) is the whole-directory admission gate skills already pass — the bundle passes the same gate.
- G-8/G-9: write-time secret/PII redaction — the exported provenance is G-9-redacted.
- L-14 emits the RRA receipt; G-2 is the replay lineage — the bundle carries both.
- P-1 owns the manifest; P-2 owns the ports the bundle carries as mechanism.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the AppBundle format + `elai app export <app>` (mechanism + manifest + RRA receipt + G-9-redacted provenance) + `elai app import <bundle>` that runs M-5 elai-verify and REFUSES on a failed receipt + mounts the mechanism.
- DEFER (Tier-2): persona regeneration against the importer's memory (needs E-5 memory + P-7 profile seam) + the friend-to-friend/marketplace sharing surface (crosses the L-12 admission + hosting seam — land with the sharing wiring pass shared with P-2 Tier-2).

## Integration
- crates/orchestrator (or knowledge): the AppBundle serializer/deserializer + the export/import flow + the M-5 verify call + the G-9 redaction pass on the provenance leg.
- CLI: `elai app export|import <target>` subcommand is the Tier-1 production caller.
- persona regen (Tier-2): reads the importer's E-5 memory namespaces + regenerates rules rows/thresholds via the P-7 profile engine.

## MOVES (wargame)
M1 define the AppBundle format (mechanism + manifest + receipt + redacted provenance; persona EXCLUDED) and the exporter that assembles it, running G-9 redaction on the provenance leg.
EXPECTED_OBSERVATION: `elai app export <app>` writes a bundle whose provenance leg has secrets/PII redacted (G-9) and whose persona layer is absent; a bundle export with an un-redactable secret in provenance is a hard refusal.
ON_FAIL: a secret leaks into the exported provenance -> the G-9 redaction wasn't on the export path -> route the provenance through G-9 before serialization, add the leak-detection test.
M2 build `elai app import <bundle>` that runs M-5 elai-verify against the bundle's receipt and REFUSES on a failed/absent receipt before mounting anything.
EXPECTED_OBSERVATION: importing a valid bundle re-verifies GREEN and mounts the mechanism (a `app_import` SSB row lands); importing a tampered bundle (mutated mechanism, stale receipt) is REFUSED with no mount.
ON_FAIL: a tampered bundle mounts -> import trusted the sender instead of re-verifying -> gate the mount on the M-5 verdict, assert the refusal in the smoke (R1 missing-evidence = FAIL).
M3 (Tier-2) regenerate the persona layer on the importer's seat against their E-5 memory, journaled + revertible.
EXPECTED_OBSERVATION: after import, the tool's thresholds/rules/tokens reflect the IMPORTER's memory, not the sender's; the regeneration is a journaled, revertible change.
ON_FAIL: the sender's persona bleeds through -> the persona wasn't excluded from the bundle -> confirm the exclusion at export (M1), regenerate from the importer's memory only.

## FORK
FORK: if M-5 elai-verify cannot yet verify a whole-app bundle (only per-changeset evidence today) -> ship Tier-1 export + import-with-receipt-presence-check (refuse on absent/malformed receipt), route full evidence re-verification to Tier-2 behind an M-5 whole-bundle capability, and record the M-5-bundle-scope seam in the decision doc. Do NOT ship a mount that skips verification entirely — presence-check is the floor, never no check.

## RECON_NEEDED
RECON_NEEDED: M-5 elai-verify's input grain (does it verify a changeset diff or can it verify a packaged artifact?) -> settle via reading M-5's shipped verifier + `grep -rn "elai-verify\|elai_verify\|M-5\|verifier" .elai_cc/crates`.
RECON_NEEDED: the G-9 redaction API's callable entrypoint for a provenance blob -> settle via `grep -rn "redact\|G-9\|secret\|PII" .elai_cc/crates` + codegraph_explore on the redaction pass.
RECON_NEEDED: where the persona layer physically lives today (which rules-rows/memory-namespaces/token files constitute "persona") -> settle via reading L-12/L-13 rules-as-data + E-5 memory namespace layout BEFORE defining the export exclusion set.

## ABORT
ABORT: if factoring an app into mechanism-vs-persona requires per-recipient CODE changes (a fork) rather than a data-only persona swap -> stop and flag; that is a substrate gap to file (the app's weirdness is in code, not data — a P-plan substrate task), never a per-recipient fork to ship (A-5 two-masters).

## WIRING_CONTRACT
produces_symbols:
  - the AppBundle format (mechanism + manifest + receipt + G-9-redacted provenance; persona excluded)
  - the `elai app export|import` CLI subcommand + the M-5-verify-gated mount + the G-9 redaction pass on export
  - the journaled `app_export` / `app_import` SSB rows
consumes_symbols:
  - P-1 AppManifest + P-2 ports, M-5 elai-verify + M-6 evidence format, L-12 admission + skillspector, G-8/G-9 redaction, L-14 RRA + G-2 lineage, E-5 memory + P-7 profile (Tier-2 persona regen)
reachable_from:
  - the `elai app export` / `elai app import` CLI subcommand on a live run  [PRODUCTION CLI]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO persona in the exported bundle (regenerated on import)
  - NO import that mounts without M-5 verification (presence-check is the floor)
  - NO unredacted provenance in an export (G-9 mandatory)
  - NO per-recipient code fork (persona is data — A-5)
observable_effect:
  trigger: "export an app, tamper the bundle's mechanism, then `elai app import` the tampered bundle"
  expect_ssb: "an app_export row on export; an app_import REFUSAL (no mount row) for the tampered bundle; an app_import mount row for the untampered one"
  expect_artifact: "the exported bundle has no persona layer + a G-9-redacted provenance; NEGATIVE: a tampered bundle that mounts, or an export leaking a secret, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai app export`/`import` subcommand dispatch calls the bundle serializer + the M-5-verify-gated mount + the G-9 redaction pass — the executor names the exact dispatch fn at RECON and wires it same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-3_persona_split_export.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = an `app_export` row landed on a real export with a persona-free, G-9-redacted bundle, and an `app_import` of a tampered bundle was refused off-agent (no mount row) while a clean bundle mounted.

INTEGRATION_SMOKE: an independent agent exports a real app carrying a planted secret in provenance (must be redacted) + a persona layer (must be excluded), then imports it on a SECOND simulated seat, tampers the mechanism between export and import (import must refuse), re-imports the clean bundle (must mount + verify), and asserts the mounted tool has NO sender persona — a full round-trip firing the redaction + refusal + persona-exclusion paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "ship a backdoor by editing the mechanism after the receipt is signed": FAILS because import re-verifies the mechanism against the receipt via M-5; a post-signing edit breaks verification and refuses the mount. Attack that SUCCEEDED and its patch — "leak the sender's API keys through the exported provenance/lineage": patched by making G-9 redaction a mandatory export-path pass (M1 stub_ban) so no secret reaches the bundle, verified by the planted-secret smoke leg.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-3 + T5 mapping + A-5 (two-masters) + A-2 (falsifier: exported-tool re-verification success rate).
- reddit r/ClaudeAI thread archive — heynoswearing "will immediately break if anyone else looks at or touches it"; Kind_Silver_1921 "someone needs to make a site to upload their best vibecoded software"; skerit "now it's actually quite trivial to fix all of these issues".
- feature-map: M-5 elai-verify, M-6 evidence format, L-12 admission + skillspector, G-8/G-9 redaction, FS-1 user-extends-via-data, L-14 RRA.

## COMPOSES-WITH
P-1 AppManifest + P-2 ports (the mechanism the bundle carries), M-5/M-6, L-12 admission, G-8/G-9, FS-1/L-13, E-5 + P-7 (Tier-2 persona regen). Re-implements NONE (GD-1). P-7 owns the profile engine persona regen calls; P-6's domain labels ride along in the ports.

End of P-3.
