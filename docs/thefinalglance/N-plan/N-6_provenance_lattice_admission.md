# N-6: provenance / trust lattice + admission-gate family — one semantic mapping across six trust scales, one scanner registry

- Track: TIER 2 (HIGH) | lever: security-composes-only-if-the-lattice-does | finding: W-19+W-29 | grounding: STRONG | nature: BUILD | effort: L | WAVE 4

- Premise: six independent "how much do we trust this byte's origin" scales, only ONE pairwise wire between them. A T2 memory row, a G-1 tainted string, and a T2 source citation are three unrelated T2s. An injection defense that loses the taint at the memory/citation/skill boundary is four fences with gaps between them. Security composes only if the lattice does.

## Steal/build
(1) declare ONE provenance lattice: a total SEMANTIC MAPPING between the six scales — NOT one shared enum. F-6 DELIBERATELY made SkillTrustTier distinct from memory-TrustTier with an auditor-enforced type barrier to avoid namespace confusion; so N-6 declares what S2 MEANS in T0-T3 terms, what taint label a T2 source implies, etc, carried on the SAME object: ingest(G-1 taint, 4 sites) -> memory(C-48 T0-T3) -> citation(H-53) -> oracle-TTL(C-30 T1-T4) -> skill-autonomy(F-6 S1-S4) -> cross-repo(K-23) -> research(4D EvidenceLevel). The current "separation for one pair, silence for the other four" is the worst of both — map all six.
(2) declare the ADMISSION family: F-1 VibeGate (code writes) + J-102 config-ingress (.md instruction surface) + J-109 learned-artifact (memory/playbook/case) + the new K-23 workspace-manifest / M-2 org-pack surfaces share ONE scanner registry (scanner legs as shared DATA, surface as parameter). They already share legs (J-102 reuses F-1's secret-grep; J-109 indexes off J-102's manifest) — just undeclared as a family. M-2 monotone-tighten + J-102 quarantine are the same verdict shape.
EXCLUDED: collapsing F-6's enum into C-48's (map, don't merge — the type barrier stays); re-implementing any scanner (the registry shares the existing legs); a seventh trust scale.

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- G-1 taint Provenance (4 ingest sites), C-48 SafeHarbor memory TrustTier T0-T3, F-6 SkillTrustTier S1-S4, C-30 oracle source_tier T1-T4, K-23 per-repo trust, 4D EvidenceLevel all exist independently (feature-map trust group).
- only J-93-C (taint-survives-memory) wires a pair; F-6 has an auditor-enforced type barrier (interlace doc W-19 refinement).
- F-1 VibeGate + J-102 config-ingress + J-109 learned-artifact admission exist; J-102 reuses F-1 secret-grep, J-109 indexes off J-102 manifest (W-29).
- E-37/E-30/E-24/G-1/F-3/J-101/J-102 assemble the injection perimeter across five letters but pass no single provenance TYPE (E note, W-19 confirmation).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the lattice MAPPING table (the six scales' total mapping) + carrying the label across the ingest->memory->citation chain + the shared scanner registry for the three EXISTING admission gates.
- DEFER (Tier-2): the K-23/M-2 new admission surfaces plugging into the registry (they land with their own units; N-6 makes the registry they plug into).

## Integration
- crates/security provenance/taint + the admission gate call sites; a lattice mapping table (frozen). F-6's type barrier is preserved.
- frontend_pair: a provenance badge on cited/memory surfaces (a W-7/N-7 render consumer), OR N/A justified.

## WIRING_CONTRACT
produces_symbols:
  - the provenance lattice mapping table (six scales, total semantic mapping)
  - the shared admission scanner registry (surface as parameter)
consumes_symbols:
  - G-1 taint / C-48 T0-T3 / F-6 S1-S4 / C-30 T1-T4 / K-23 repo trust / 4D EvidenceLevel
  - F-1 VibeGate secret-grep leg / J-102 manifest / J-109 admission (as registry members)
reachable_from:
  - a real ingest -> memory-write -> citation chain  [PRODUCTION data-plane seam]
  - a write to any admission surface  [PRODUCTION admission seam]
stub_ban:
  - NO collapse of F-6's enum into C-48's (map, don't merge)
  - NO taint lost at the memory/citation/skill boundary
  - NO re-implementation of a scanner leg (share via the registry)
  - NO seventh trust scale
frontend_pair: provenance badge on cited/memory surfaces (via N-7) OR "N/A justified"
observable_effect:
  trigger: "an ingest of a web-tainted string -> memory-write -> citation"
  expect_ssb: "the taint/trust label MAPS coherently end-to-end (rides the existing G-1/provenance rows; +admission_scanned if the shared registry emits)"
  expect_artifact: "the trust label at citation is the lattice-mapped image of the ingest label; NEGATIVE: a tainted string that loses its taint at the memory/citation/skill boundary must FAIL"

## Grounding
- interlace doc W-19 (six trust scales) + its F-6 refinement (map-don't-merge), W-29 (admission family), TIER-2 item 6; E note (injection perimeter across five letters).
- CLAUDE.md security posture + wiring-enforcement (fail-closed admission).

## COMPOSES-WITH
G-1/C-48/F-6/C-30/K-23/4D (the six scales), F-1/J-102/J-109 (admission family), J-93-C (the existing pair). Re-implements NONE (GD-1). Security composes only if the lattice does; this is the security-grade version of the one-claim-object unification.

End of N-6.
