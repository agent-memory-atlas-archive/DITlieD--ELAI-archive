# I-40: trust-layer wrapper + RRA spec — open .rra format, external-agent proxy, two-tier wedge strategy

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever WEDGE | grounding STRONG | nature BUILD | effort L | WAVE 5

- Premise: the ELAI trust substrate (taint tagging, write-gates, RRA emission, attestation) wraps only
  ELAI's own generator loop. The competitive wedge is to make that layer available as an open wrapper
  around any external agent, publishing the .rra format as a conformance-checkable spec. G-9 redaction
  is the publication precondition: an exported RRA must not carry raw secret bytes. Both competitive
  reviews converge on this as the wedge strategy.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Steal/build
Two deliverables:

(a) .rra OPEN SPEC: docs/spec/rra-spec-v1.md — the replayable-evidence artifact format published as a
versioned, in-repo specification. Includes the I-18 per-row chain_prev_hash structure (the chained format
the spec documents), the taint-flow payload schema, the write-gate record schema, and the RRA bundle
envelope. A conformance checker (simple CLI: `elai rra verify <path>`) validates that a bundle satisfies
the spec. The spec is versioned so external implementers can target it.

(b) MINIMAL PROXY WRAPPER: `elai wrap -- <agent cmd>` intercepts an external agent's tool traffic at
the MCP/stdio layer, runs each tool call through:
  - G-1 taint tagging on any untrusted ingest
  - F-1/F-9/G-3 write-gates on destructive tool classes
  - RRA row emission per tool call (the same rra.rs surface as ELAI's own loop)
  - G-9 redaction before export

The wrapper is minimal: it composes the shipped trust substrate (GD-1), adds no new trust primitives.
The point is to prove the gates bite outside ELAI's own generator loop. External-agent fixture harness
tests ONE fixture agent; the full external-agent matrix (multiple real MCP server implementations)
is DEFERRED (Tier-2) — it requires real MCP server instances beyond a single controllable fixture.

TWO-TIER STRATEGY recorded in the spec (section 1): wrapper = wedge + standard track; the deep ELAI
orchestrator with full wiring stays the premium destination. The wrapper is the on-ramp, not the product.

G-9 redaction is a PRECONDITION: the wrapper MUST call G-9's redaction layer before emitting the RRA
bundle. An unredacted exported RRA is a stub_ban violation.

## Existing ground truth
- IS-10 (grounding map): rra.rs build_rra:384 + verify_rra, attestation.rs exists — G-2/G-8 shipped (GD-1; the wrapper composes this, never forks)
- IS-11 (grounding map): capability_flow.rs taint_text:234, safety/src/provenance.rs:131 — G-1 taint substrate (GD-1)
- PLAN-asserted: G-2 .rra format (the format the spec documents); executor re-greps per W-7
- PLAN-asserted: F-1/F-9/G-3 write-gates (the gate substrate the wrapper applies); executor re-greps per W-7
- PLAN-asserted: G-9 redaction layer (the publication precondition); executor re-greps per W-7
- PLAN-asserted: B-37 adapter patterns (the MCP/tool dispatch precedent); executor re-greps per W-7
- PLAN-asserted: E-24 MCP bridge hardening (the proxy layer pattern); executor re-greps per W-7
- I-18 dependency: the per-row chain_prev_hash structure the spec documents (I-18 must ship first or in Wave 5 parallel)

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- docs/spec/rra-spec-v1.md (~150 LOC): the .rra format spec, versioned, with the two-tier strategy section
- `elai rra verify <path>` CLI: conformance checker (~100 LOC) — validates a bundle against the spec
- `elai wrap -- <agent cmd>` CLI subcommand: the minimal proxy (~250 LOC) — taint + write-gate + RRA emit + G-9 redaction, ONE fixture agent tested end-to-end
- The wrapper reuses taint_flow + run_artifact_emitted SSB rows (GD-1 — NO new action_type for I-40)
- Wrapped-run rows render in the existing RRA viewer (frontend_pair via the existing viewer)
- decision doc: workflow/reasoning/I-40-decisions.md

DEFER (Tier-2 / gated):
- Full external-agent matrix (multiple real MCP server implementations beyond one fixture agent) — needs real MCP servers; user-approval gated as a new-network-dep integration
- Commercial wrapper packaging / distribution — business decision, post-wedge validation
- Sigstore/Rekor push for external-agent RRAs — I-18 is the prerequisite; compose after I-18 ships

## Integration
- new docs/spec/rra-spec-v1.md (~150 LOC in repo docs/spec/)
- new crates/cli/src/wrap_cmd.rs (~250 LOC): the minimal proxy subcommand
- new crates/cli/src/rra_verify_cmd.rs (~100 LOC): conformance checker
- seam to IS-10 rra.rs build_rra:384 (the RRA emission target; GD-1)
- seam to IS-11 capability_flow.rs taint_text:234 (taint tagging; GD-1)
- seam to F-1/F-9/G-3 write-gates (the gate application layer; GD-1)
- seam to G-9 redaction (the pre-export filter; GD-1 — called before any emit)
- seam to E-24 MCP bridge hardening (the proxy layer pattern; GD-1)
- frontend_pair: wrapped-run rows render in the existing RRA viewer — no new Svelte component; Tauri read path for the existing viewer (cross_lang_diff.py PASS if the RRA viewer has a Svelte counterpart; verify at executor re-grep W-7)

## WIRING_CONTRACT
produces_symbols:
  - docs/spec/rra-spec-v1.md (the open .rra format spec, versioned)
  - crates/cli/src/wrap_cmd.rs (the minimal proxy subcommand)
  - crates/cli/src/rra_verify_cmd.rs (the conformance checker)
  - NO new SSB action_type — reuses taint_flow + run_artifact_emitted rows (GD-1; I-0 delta table note)
consumes_symbols:
  - IS-10 rra.rs build_rra:384 + verify_rra (the RRA emission substrate; GD-1 — compose, never fork)
  - IS-11 capability_flow.rs taint_text:234 (taint tagging on untrusted ingests; GD-1)
  - F-1/F-9/G-3 write-gates (the destructive-tool gate layer; GD-1)
  - G-9 redaction (the pre-export filter; GD-1 — mandatory, not optional)
  - E-24 MCP bridge hardening (the proxy pattern; GD-1)
  - B-37 adapter patterns (the dispatch pattern; GD-1)
  - I-18 per-row chain_prev_hash (the chained format the spec documents; compose after I-18 or in parallel if I-18 ships first)
reachable_from:
  - `elai wrap -- <agent cmd>` CLI -> wrap_cmd::run -> taint + gate + rra.rs emit + G-9 redact -> taint_flow + run_artifact_emitted SSB rows  [PRODUCTION wrap surface]
  - `elai rra verify <path>` CLI -> rra_verify_cmd::verify -> spec conformance check  [PRODUCTION verify surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO exporting an unredacted RRA (G-9 redaction is the precondition; an RRA bundle with raw secret bytes is a P0 pre-publication)
  - NO new trust primitives in the wrapper (the wrapper composes shipped substrate — GD-1; adding new gate logic is an unsolicited refactor)
  - NO external-agent matrix beyond one fixture agent in Tier-1 (the full matrix is Tier-2 DEFERRED; testing a mock fixture is the Tier-1 scope)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  wrapped-run rows render in the existing RRA viewer — no new Svelte component; Tauri read via the existing viewer path (verify at executor re-grep W-7 whether the RRA viewer has a live Svelte pair; cross_lang_diff.py PASS if so).
observable_effect:
  trigger: "elai wrap -- <fixture-agent cmd> (the fixture agent attempts an untrusted ingest + a seeded malicious write)"
  expect_ssb: "taint_flow rows on the untrusted ingest (taint propagates through the proxy); a write-gate BLOCK row on the seeded malicious write (the wrapper proves the gates bite outside ELAI's own loop); the RRA bundle emitted carries G-9 redaction (no raw secret bytes in the export)"
  expect_artifact: "elai rra verify <bundle_path> PASSES on the wrapper-produced bundle; the conformance suite REJECTS a tampered artifact (R2 — the conformance checker exits nonzero); wrapped-run rows render in the existing RRA viewer; G-9 redaction verified: no raw secret bytes in the exported bundle"

## Grounding
- IS-10 (grounding map): rra.rs build_rra:384 + attestation.rs — confirms COMPOSE not BUILD-from-scratch
- IS-11 (grounding map): taint_text:234 + safety/src/provenance.rs:131 — taint substrate confirmed shipped
- IC-16 (grounding map): "trust spine is substantially wired" — the I-40 wrapper composes this, confirms GD-1 posture
- Competitive reviews (I-0 source synthesis): both converge on the wrapper-as-wedge strategy
- I-18 (WAVE 5 predecessor): the per-row chain_prev_hash format the spec documents
- G-9 redaction (PLAN-asserted; executor re-greps per W-7): the publication precondition
- Trust-review A5 / B1 / B7: the wedge / wrapper / standard-track sequencing
- SSB: NO new action_type — reuses taint_flow + run_artifact_emitted (I-0 delta table; re-read at edit W-9) — baseline 221/532 as of 2026-06-10

## COMPOSES-WITH
- IS-10 rra.rs / attestation.rs — the RRA emission substrate; do NOT re-implement the RRA format
- IS-11 capability_flow.rs — the taint tracking; do NOT re-implement taint propagation
- F-1/F-9/G-3 write-gates — apply as-is; do NOT fork gate logic
- G-9 redaction — call before any emit; do NOT re-implement redaction logic
- E-24 MCP bridge hardening — the proxy pattern; mirror, do NOT re-implement the bridge
- I-18 row chaining — the chained format the spec documents; compose after I-18 ships

End of I-40.
