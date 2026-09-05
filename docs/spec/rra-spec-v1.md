rra-spec-v1 — Reproducible-Run Artifact (.rra) Open Format Specification

Version: 1  (format_version=1 in every bundle)
Status: STABLE — I-40 initial publication
Producer: ELAI trust substrate (crates/orchestrator/src/rra.rs)
Consumers: elai rra verify, elai rra show, elai replay --verify, RraViewerPanel.svelte

---

1. TWO-TIER STRATEGY

The .rra format is the open standard track for the ELAI trust wrapper wedge. Any
conforming .rra bundle — whether produced by ELAI's own generator loop or by the
`elai wrap -- <agent cmd>` proxy — can be submitted to `elai rra verify` for
spec-conformance validation and to `elai replay --verify` for content-hash
re-verification.

Tier 1 (this spec, open): publish the .rra format as a versioned, in-repo
specification so external implementers can produce conformant bundles and
interoperability tools can validate them. The wrapper is the on-ramp.

Tier 2 (deep ELAI orchestrator, premium): the full ELAI generator loop with live
taint tagging, write-gates, per-row SSB persistence, and Sigstore/Rekor push. The
premium destination for teams requiring continuous trust evidence.

---

2. FORMAT OVERVIEW

A .rra bundle is a single file containing a msgpack-encoded RunArtifact struct
(crates/orchestrator/src/rra.rs::RunArtifact). The bundle is stored UNCOMPRESSED
(no zstd frame; a compression envelope is a deferred enhancement per
RRA_COMPRESSION_DEFERRED). The Rust msgpack codec is rmp-serde.

Field order and presence are defined by the RunArtifact struct layout at the time
of writing. A conformance checker validates REQUIRED fields are present and
non-empty; it does not enforce field ordering (msgpack maps are unordered).

---

3. BUNDLE SCHEMA

3.1 Top-level RunArtifact

  format_version   : u32   REQUIRED — must equal 1 for bundles conforming to this spec.
  session_id       : String REQUIRED — opaque session identifier; must be non-empty.
  seed             : u64    REQUIRED — the ELAI_SEED value used for this run.
  hardware_class   : String REQUIRED — one of "cpu", "cuda", "metal", "mps", "unknown".
  system_fingerprint_pin : Option<String> — cloud model fingerprint if known; None allowed.
  model_pin        : Option<String> — model identity if recovered from the stream; None allowed.
  guarantee_level  : GuaranteeLevel REQUIRED — see section 4.
  events           : Vec<RraEvent> REQUIRED — the ordered SSB action stream; may be empty for
                     a session that produced no rows (not an error, but a semantically weak bundle).
  manifest         : Vec<GateManifestEntry> — F-gate verdict hashes present in the stream;
                     may be empty when all gates were off (gate-off state, not a failure).
  root_blake3      : String REQUIRED — 16-hex prefix blake3 Merkle root over the canonical
                     bundle bytes; must be non-empty. See section 6.
  chain_prev_hash  : Option<String> — I-18: 16-hex blake3 of the PREVIOUS SSB row's canonical
                     bytes; None = GENESIS (first bundle in this SSB path).
  stack_hash       : Option<StackHash> — I-18: 5-layer blake3 StackHash; None on pre-I-18 bundles.
  objective_hash   : Option<String> — I-45: 16-hex blake3 of the sealed task objective; None
                     on sessions without an IdeaFramingStarted row.

3.2 RraEvent

  seq          : i64    REQUIRED — monotonically increasing SSB sequence number.
  action_type  : String REQUIRED — the action_type SQL string for this event (e.g. "taint_flow").
  payload_bytes: Vec<u8> REQUIRED — the raw msgpack payload bytes for this event.
  event_blake3 : String REQUIRED — 16-hex blake3 content hash (blake3 over seq || action_type || NUL || payload_bytes); used in tamper check.

3.3 GateManifestEntry

  ssb_event_seq   : i64    REQUIRED — seq of the F-gate row this entry covers.
  gate            : String REQUIRED — gate identifier (e.g. "F-1", "F-4", "F-9", "F-10").
  action_type     : String REQUIRED — action_type string for this gate row.
  verdict_blake3  : String REQUIRED — 16-hex blake3 of the gate verdict field.

3.4 StackHash (I-18, 5-layer)

  state_hash   : String REQUIRED — blake3 over the session seed + hardware_class.
  deps_hash    : String REQUIRED — blake3 over the model_pin + system_fingerprint_pin.
  process_hash : String REQUIRED — blake3 over the events stream bytes.
  result_hash  : String REQUIRED — blake3 over the gate manifest entries.
  verify_hash  : String REQUIRED — blake3 combining all four layers + the session_id (the combiner).

All StackHash fields are 16-hex blake3 prefixes. All layers use blake3
(no sha256/blake2; hard rule per ELAI codebase policy).

3.5 GuaranteeLevel

One of (msgpack-encoded as snake_case strings):
  reproducible_tokens        — bit-identical token replay. NOT produced by build_rra (honest scope).
  reproducible_action_stream — action stream + every present F-gate verdict hash re-derives.
  verify_divergence_only     — stream + present hashes are re-checkable; token identity not provable.

A conformant producer MUST NOT emit reproducible_tokens for bundles produced by
the trust-layer wrapper (the cloud lane's system_fingerprint is opaque; GD-5).

---

4. GUARANTEE LEVEL SEMANTICS

verify_divergence_only is the honest ceiling for ELAI cloud-lane runs and for the
`elai wrap` proxy (no system_fingerprint pin + no bit-identical token replay).
reproducible_action_stream is achievable when a local-lane model pin is known and
the ELAI_SEED pin is set. The conformance checker accepts all three levels; the
RRA viewer renders the level as the trust badge.

---

5. TAINT-FLOW PAYLOAD SCHEMA (SSB action_type = "taint_flow")

The payload is a TaintFlowPayload msgpack struct:
  kind         : TaintFlowKind (one of "web", "tool_output", "file", "sandbox")
  origin_blake3: String — 16-hex blake3 of the origin bytes that triggered the taint
  trust_level  : String — the UntrustedTaint verdict label ("suspect" or "malicious")
  sink_tool    : Option<String> — the tool that attempted to consume the tainted data; None if blocked at ingest
  blocked      : bool — true if a write-gate blocked the tainted sink
  task_id      : String — opaque task identifier

The trust-layer wrapper emits a taint_flow row for every untrusted ingest it
intercepts (taint_text() verdict non-Clean). The sink_tool field identifies
the destructive write class the wrapper blocked.

---

6. WRITE-GATE RECORD (embedded in GateManifestEntry)

A bundle produced by the `elai wrap` proxy includes GateManifestEntry rows for
every destructive-write-class tool call that the F-1/F-9/G-3 gate evaluated. The
verdict_blake3 field is the 16-hex blake3 of the gate HookFinding message string.
A Block verdict (HookVerdictKind::Block) means the destructive tool call was
rejected by the gate before it reached the external agent.

---

7. RRA BUNDLE ENVELOPE

The bundle is the msgpack encoding of a RunArtifact value (top-level object, not
an array). No magic bytes or header framing beyond the msgpack encoding itself. A
conformance checker can distinguish a .rra from arbitrary binary by:
  (a) decoding as msgpack map,
  (b) checking format_version == 1,
  (c) checking root_blake3 is a non-empty string of length >= 16.

File extension: .rra (RECOMMENDED; not enforced by the checker).

---

8. CONFORMANCE REQUIREMENTS (what `elai rra verify` checks)

REQUIRED (exit nonzero if absent or invalid):
  C1. format_version == 1.
  C2. session_id is non-empty.
  C3. root_blake3 is non-empty.
  C4. Every RraEvent has non-empty action_type and non-empty event_blake3.
  C5. Every GateManifestEntry has non-empty gate, action_type, and verdict_blake3.
  C6. The bundle decodes from msgpack without error (a truncated or bit-flipped
      bundle is a conformance failure, not a warning).

INFORMATIONAL (printed; does not affect exit code):
  I1. chain_prev_hash present (indicates row chaining is active).
  I2. stack_hash present (indicates I-18 5-layer hash is active).
  I3. objective_hash present (indicates task-objective sealing is active).
  I4. events is non-empty (an empty event list is valid but weak).

DISTINCTION FROM `elai replay --verify`:
  `elai rra verify` checks STRUCTURE/SCHEMA (this spec, above).
  `elai replay --verify` checks CONTENT (recomputes blake3 root + event content
  hashes; reports Verified / Divergence / Tampered). Both are independent; a bundle
  can be schema-conformant but content-tampered, or content-intact but schema-old.

---

9. REDACTION PRECONDITION (G-9, mandatory for all exported bundles)

A .rra bundle MUST NOT carry raw secret bytes in any payload field. The ELAI
producer calls redact_payload() on every SSB payload BEFORE write_rra(); the
`elai wrap` proxy enforces the same precondition before emitting the bundle. An
exported .rra that contains a raw API key, password, or other secret is a P0
violation; the redaction layer emits a SecretRedacted advisory row and replaces
the secret with a stable token (e.g. "[REDACTED_AWS_KEY]").

---

10. I-18 ROW CHAINING

When chain_prev_hash is non-None, the producer computed a per-row blake3 chain
linking this bundle to the PREVIOUS SSB row in the session stream. The genesis
bundle (first RRA in a session) has chain_prev_hash == None. A verifier can
walk the chain backwards from the latest bundle to the genesis to confirm no row
was inserted or deleted. The chain link is NOT part of the C1-C6 conformance
checks (pre-I-18 bundles without chain_prev_hash are still spec-conformant); it
is surfaced as I1 above.

---

11. EXTERNAL-IMPLEMENTER NOTES

A third-party tool that produces .rra bundles conformant with this spec can submit
them to `elai rra verify <path>` for spec-conformance validation. The tool MUST:
  - Encode the bundle as msgpack (rmp-serde compatible encoding).
  - Set format_version = 1.
  - Provide a valid root_blake3 (16-hex blake3 over the canonical bundle bytes, per
    the compute_root function in rra.rs; see source for the canonical byte derivation).
  - Not include raw secret bytes (see section 9).

The Tier-1 conformance check (C1-C6) validates these requirements. The Tier-2
`elai replay --verify` content-hash check additionally requires the events array
to carry the same payload bytes that the canonical SSB stream logged.
