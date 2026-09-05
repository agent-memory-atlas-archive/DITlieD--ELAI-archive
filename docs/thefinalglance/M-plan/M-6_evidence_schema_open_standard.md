# M-6 — Evidence format as a versioned open standard (the schema, not just the verifier)
Track M | lever REACH + AUDITABLE | grounding STRONG (spec-eval 2026-07-05 §5 ADD-1: a solo author
cannot win a plugin-ecosystem war (L3) but can own the FORMAT agent evidence is exchanged in — the
SLSA/in-toto precedent is format-first, tool-second [training, unsearched — re-verify at execution])
| nature BUILD-compose + docs | effort M | WAVE M3 | depends M-5a (elai-verify extracted),
G-2/G-8/K-6 emit paths stable.
PROOF_SPINE: none directly (distribution amplifier for the spine's numbers — it makes PS artifacts
consumable by third parties; user-acked 2026-07-05 as spine-exempt).

## Objective
M-5 open-sources the verifier BINARY; M-6 publishes what the binary verifies: a versioned,
machine-readable EVIDENCE FORMAT specification ("EEF v1" — ELAI Evidence Format; final name at
execution) covering the RRA bundle, the attestation predicate, the compliance bundle, and the
claim-receipt row shapes — so third-party tools can EMIT ELAI-verifiable evidence and elai-verify
becomes the reference verifier of a format rather than the private tool of one engine. This converts
the moat from "our binary" to "our grammar": the one ecosystem play a single-author project can
defend, and the counter-move K-0's kill-switch already anticipates ("GitHub ships native evidence ->
pivot to be-compatible" — publishing first makes ELAI the thing to be compatible WITH).

## Slices
(a) SCHEMA EXTRACTION [the one build]. Generate the spec's machine half FROM THE SHIPPED TYPES —
    schema files (JSON-Schema for JSON surfaces; a documented msgpack layout for the RRA) derived
    from the same Rust structs the engine serializes (schemars-class derivation; verify the crate
    per anti-slopsquatting at execution), never hand-written twice. Covers: the RRA bundle (G-2
    action stream + seed + hardware class + fingerprint + evidence manifest + blake3 root), the
    in-toto v1 attestation predicate (G-8), the K-6 compliance bundle, and the verdict/receipt row
    shapes K-3/K-11 render (gate verdict, stdout-hash-bound test outcome, DSR band, anchored
    finding). One source of schema truth = the code.
(b) PROSE SPEC + CONFORMANCE FIXTURES. docs/strategy/eef/ (or a sibling repo at publication): the
    versioned prose spec (field semantics, integrity rules — what a verifier MUST check, what
    missing evidence MUST mean: fail, per R1), plus a conformance fixture pack (valid bundles +
    one-byte-tampered + missing-field negatives). `elai-verify conform <artifact>` (an M-5 binary
    subcommand extension) validates any artifact against a pinned spec version.
(c) VERSIONING + GOVERNANCE. Append-only field evolution locked by a golden file (the E-45
    action-type golden-lock pattern applied to the schema); every emitted artifact carries
    spec_version; a breaking change requires a major version + a migration note; the golden lock
    lives in the frozen gate-manifest set (R8) so a run cannot mutate the format it is judged in.
(d) PUBLICATION + THIRD-PARTY EMITTER STORY. Publish spec + schemas + fixtures alongside the
    elai-verify repo (same license family as M-5a; user picks); a short "emit EEF from your tool"
    doc with one worked example (a minimal third-party emitter producing a conformant verdict row).
    Claims discipline: it is a "published format" until >=1 EXTERNAL emitter exists; the words
    "open standard" / "industry standard" are KD-1-gated on that adoption evidence (the K-1 lint
    gains an adoption-claim recognizer).

## LOCKs
- LOCK-1 ONE SOURCE OF SCHEMA TRUTH (GD-1): the machine schemas are DERIVED from the engine's
  serialization types; a hand-forked schema = drift = the exact failure attestation exists to
  prevent (mirrors M-5 LOCK-2).
- LOCK-2 KD-1 ON ADOPTION: no "standard" claim without an external emitter; no ecosystem-size
  claim ever; every comparison figure [EXTERNAL].
- LOCK-3 the ENGINE STAYS CLOSED (GTM record): only the format, fixtures, and verifier are open.
  Publishing the schema must not leak engine internals beyond what emitted artifacts already show;
  the G-9 redaction guarantees are restated in the spec as emitter REQUIREMENTS.
- LOCK-4 FAIL-CLOSED SEMANTICS ARE NORMATIVE: the spec text makes R1 (missing evidence = FAIL,
  UNVOUCHED never renders clean) a conformance requirement for verifiers, not a recommendation —
  a permissive third-party verifier is non-conformant by definition.

## COMPOSES-WITH (do NOT reimplement)
M-5a elai-verify (gains `conform`; the reference verifier), G-2 RRA + G-8 attestation + K-6 bundle
(the emit paths — the schemas describe them, never fork them), E-45 golden-lock pattern (schema
evolution), K-1 claims lint (adoption-claim recognizer extension), K-3/K-11 (the verdict/receipt
shapes rendered into PRs), gate-manifest R8 (the golden lock joins the frozen set).

## WIRING_CONTRACT
produces: docs/strategy/eef/ spec v1 (prose + derived schemas + conformance fixtures), the
  `elai-verify conform` subcommand, the schema golden-lock file (added to .claude/gate-manifest.txt),
  the K-1 lint adoption-claim recognizer, the third-party emitter example.
consumes: the engine serialization types (derivation source), G-2/G-8/K-6 emit paths, M-5a binary,
  E-45 golden-lock pattern, K-1 lint.
reachable_from: `elai-verify conform <artifact>` on an engine-free machine (real entrypoint); the
  golden lock enforced by gate_freeze_check.py on every run (R8 path).
trigger: on a CLEAN machine: `elai-verify conform fixtures/run.rra && elai-verify conform
  fixtures/tampered.rra; test $? -ne 0`
expect_ssb: none (standalone verifier + docs; E-23-class legs journal nothing).
observable_effect (done-gate): a conformant fixture PASSes and a one-byte-tampered / missing-field
  fixture FAILS on an engine-free machine (R2); the derived schemas byte-match a regeneration from
  the engine types (LOCK-1 drift test); a schema field removal trips the golden lock (R8); a seeded
  "industry standard" phrase in the spec/README FAILS the K-1 lint until the adoption row exists
  (LOCK-2 arm); the worked third-party example emits a row `elai-verify conform` accepts.

## Risks (top 3)
R1 premature standardization: freezing v1 before the RRA/attestation surfaces stabilize forces a
  fast v2 and burns credibility. Mitigation: M-6 sits in WAVE M3 behind GATE M-0 (the numbers +
  emit paths exist first); v1 scope = only artifacts elai-verify already verifies today.
R2 the schema leaks engine detail: review pass against LOCK-3 with the G-9 redaction spec; the
  conformance fixtures are generated from redacted runs only.
R3 nobody adopts (the honest default): the format still pays for itself as internal rigor (the
  golden lock + conform subcommand harden ELAI's own emit paths) — the wedge costs nothing extra;
  adoption claims stay gated so non-adoption never becomes a false claim.
