rra-spec-v1 — Reproducible-Run Artifact bundle format specification

Format version: 1 (RRA_FORMAT_VERSION = 1, crates/orchestrator/src/rra.rs)
Implementation: crates/orchestrator/src/rra.rs + crates/cli/src/rra_verify_cmd.rs
CLI surface: `elai rra verify <path>` (C1-C6), `elai replay --verify` (GD-5 content divergence)

NOTE: the canonical source of truth for field definitions is crates/orchestrator/src/rra.rs.
This file documents the structural requirements that rra_verify_cmd.rs enforces as C1-C6.

3. Bundle structure (RunArtifact — msgpack serialized)

Required top-level fields:
  format_version: u32          — must equal 1 (C1)
  session_id: String           — the session whose stream is bundled; must be non-empty (C2)
  seed: u64                    — pinned ELAI_SEED the stream re-derives under
  hardware_class: String       — snake-case label: "cpu" / "gpu:<sku>" / "unknown"
  system_fingerprint_pin: Option<String>  — cloud-lane opacity marker; None on local runs
  model_pin: Option<String>    — latest model_swap.to_model; None when no swap recorded
  guarantee_level: GuaranteeLevel  — honest per-lane guarantee level (GD-5 enum)
  events: Vec<RraEvent>        — ordered action stream (see section 4)
  manifest: Vec<GateVerdictHash> — F-gate evidence manifest (see section 5)
  root_blake3: String          — must be non-empty; blake3 root over canonical bundle bytes (C3)

Optional fields (None on LEGACY receipts, #[serde(default)]):
  objective_hash: Option<String>  — I-45: blake3 16-hex of canonical task objective bytes
  chain_prev_hash: Option<String> — I-18: rolling per-row chain hash (previous row's bytes)
  stack_hash: Option<StackHash>   — I-18: 5-layer blake3 stack hash (STATE/DEPS/PROCESS/RESULT/VERIFY)

4. RraEvent structure

Each element of events:
  seq: i64           — append-only sequence number (structural ordering)
  action_type: String — SSB action_type wire tag; must be non-empty (C4)
  payload: Vec<u8>   — raw msgpack payload bytes (untruncated, faithful stream)
  event_blake3: String — content hash SEALED at build time; must be non-empty (C4)

Content hash formula (C4 recomputation):
  blake3(seq.to_le_bytes() || action_type.as_bytes() || NUL || payload)
  truncated to first 16 hex characters

5. GateVerdictHash structure (manifest entries)

Each element of manifest:
  gate: String           — gate identifier; must be non-empty (C5)
  action_type: String    — SSB action_type for this verdict; must be non-empty (C5)
  ssb_event_seq: i64     — sequence number of the SSB row this verdict covers
  verdict_blake3: String — blake3 of the verdict bytes; must be non-empty (C5)

6. Root hash (C3)

root_blake3 = blake3 over canonical_bundle_bytes, truncated to 16 hex characters.
canonical_bundle_bytes is the ordered concatenation of the bundle's serialized fields
(implementation: crates/orchestrator/src/rra.rs::compute_root). A single-byte tamper
anywhere in the bundle changes root_blake3; `elai replay --verify` re-derives it.

7. StackHash (I-18, optional)

Five layers, each a blake3 16-hex prefix:
  state:   blake3 of concatenated SSB entry bytes (any row mutation changes this)
  deps:    blake3 of Cargo.lock bytes at seal time (dependency pin)
  process: blake3 of seed || hardware_class || system_fingerprint_pin
  result:  blake3 of the G-2 bundle root_blake3
  verify:  blake3 of (state || deps || process || result) — cross-layer integrity

A Cargo.lock mutation mid-run causes build_rra to return RraError::CargoLockMutated.
All five layers use blake3 (never sha256/blake2).

8. Required conformance checks (C1-C6)

C1: format_version == 1
C2: session_id is non-empty
C3: root_blake3 is non-empty
C4: every RraEvent has non-empty action_type AND non-empty event_blake3
C5: every GateVerdictHash has non-empty gate, non-empty action_type, non-empty verdict_blake3
C6: msgpack decode succeeds without error (implied by reaching C1-C5)

Informational items (I1-I4) — checked by rra_verify_cmd, do not affect exit code:
  I1: chain_prev_hash is present (I-18 row chaining active)
  I2: stack_hash is present (I-18 5-layer hash active)
  I3: objective_hash is present (I-45 task-objective sealing active)
  I4: events list is non-empty

A C1-C6 failure causes `elai rra verify` to exit nonzero (CI gatable).
`elai replay --verify` is a separate orthogonal check (GD-5 content divergence — not structural).
