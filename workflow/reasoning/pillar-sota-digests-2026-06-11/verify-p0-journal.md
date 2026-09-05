verify-p0-journal.md
task_id: verify-p0-journal
date: 2026-06-11
scope: web research (read-only) — primary source verification of 6 hardening claims for SSB/RRA journal

---

## 1. RFC 6962 / RFC 9162 Merkle Inclusion + Consistency Proofs

VERDICT: VERIFIED
URLS:
  https://datatracker.ietf.org/doc/html/rfc6962
  https://www.rfc-editor.org/rfc/rfc9162.html

FACTS CONFIRMED:
  - O(log n) audit path: RFC 6962 Section on audit paths states "the number of nodes in the resulting
    proof is bounded above by ceil(log2(n)) + 1." Confirmed O(log n), not O(n).
  - Per-event vs batched: RFC 6962 says a log SHOULD sign the tree hash "every time it appends new
    entries" but MUST produce a Signed Tree Head no older than the Maximum Merge Delay (MMD). The
    design explicitly allows batching many entries into one signed tree head; this is the standard
    CT log implementation pattern. Batched Merkle checkpointing (sign the tree root every ~N events,
    not per-event) is confirmed standard practice; the "~850x slower per-event" claim in the report
    is directionally sound (adding an Ed25519 sign per row vs one sign per batch of thousands would
    easily account for this cost) though the exact multiplier is not verifiable from the RFC alone.
  - RFC 9162 (CT v2) adds InclusionProofDataV2 with log_id, tree_size, leaf_index, inclusion_path.
    Consistency proofs between two Signed Tree Heads also confirmed.

WHAT THIS MEANS FOR ELAI:
  Our I-18 plan already has a per-row blake3 chain (chain_prev_hash) but no periodic signed tree head.
  RFC 6962-style batched Merkle checkpointing — build a tree of batch_size event hashes, sign its
  root with Ed25519, store the signature — would let a verifier confirm any single row is in the log
  with O(log n) proof path instead of replaying the full O(n) chain. This is the specific gap:
  I-18 chains rows but does not produce compact inclusion proofs. Adding a checkpoint every 1000 rows
  is a small rider on top of I-18's chain_prev_hash, not a replacement.

NEW vs ALREADY-PLANNED: RIDER-ON-EXISTING (I-18)
  I-18 plans chain_prev_hash + 5-layer stack hash. It does NOT plan periodic Merkle checkpoints or
  inclusion proof generation. Batched checkpointing is a real, small additive improvement on top of
  I-18 Tier-2 — but only if verification at scale becomes a requirement. At current SSB volumes
  (thousands of rows per run, not millions) O(n) replay is fine.

---

## 2. arXiv:2602.20214 PunkGo / punkgo-kernel

VERDICT: VERIFIED (paper exists, repo exists, stars confirmed low)
URLS:
  https://arxiv.org/abs/2602.20214
  https://github.com/PunkGo/punkgo-kernel
  https://docs.rs/punkgo-kernel/latest/punkgo_kernel/

FACTS CONFIRMED:
  - Paper title: "Right to History: A Sovereignty Kernel for Verifiable AI Agent Execution" by Jing Zhang.
    Published on arXiv at 2602.20214. Real paper, real arXiv ID.
  - Repo: github.com/PunkGo/punkgo-kernel — Rust (100%), v0.5.1 released 2026-03-16, 3 GitHub stars.
    Active development with Windows compat fixes in the latest release.
  - Crate: docs.rs/punkgo-kernel — deps include tokio, sqlx, ed25519-dalek, tlog_tiles (RFC 6962 impl).
    54.27% documented. Explicitly experimental.
  - Design: "committer not a judge" — kernel is the sole linearization point / write path. Agents
    submit Directive structs; the kernel validates, records (seven-step pipeline: validate, quote energy,
    reserve, check payload, settle, append to Merkle tree, return receipt), and returns a receipt.
    Agents do not write directly to the log. This IS writer isolation at the journal level.
  - Performance: sub-1.3ms median, ~400 actions/sec, 448-byte Merkle inclusion proofs at 10,000 entries.
  - NOT production-ready: 3 stars, 54% docs coverage, explicitly research prototype status.

COMPARISON TO I-1 (directive-supervisor):
  I-1 is about moving SIDE-EFFECT EXECUTION out of the generator's address space — the Supervisor is the
  sole executor of tool calls (file writes, shell commands, network). PunkGo's writer isolation is
  narrower and orthogonal: it isolates LOG WRITES specifically (agents cannot write to the audit log
  directly, only the kernel can). These are different isolation boundaries:
  - I-1: "the generator cannot call any sink (file system, network, shell)"
  - PunkGo: "the generator cannot write to the audit log"
  PunkGo's isolation is more specifically about journal integrity than general capability isolation.
  Our journal is a single-writer SQLite (ssb.rs single-writer documented at ssb.rs:5) which already
  enforces the same property structurally — only the orchestrator process writes to the SSB. The SSB
  WAL is not exposed as a multi-writer API. So the writer-isolation property PunkGo formalizes is
  already the STRUCTURAL REALITY of our SSB design; it is not explicitly documented as a security
  invariant or contract-tested as such.

NEW vs ALREADY-PLANNED: ALREADY-PLANNED (structurally, via SSB single-writer design)
  The writer-isolation invariant is real and worth a contract test, but the mechanism already exists.
  The gap is documentation + a contract test that asserts the invariant, not a new architectural
  component. Adding one contract test to assert "no code path outside ssb.rs write_payload can insert
  an SSB row" is a small addition, potentially a I-18 rider.

---

## 3. Sigstore Rekor as Offline-Verifiable Anchor

VERDICT: VERIFIED (offline stapled inclusion proofs are a first-class Sigstore feature)
URLS:
  https://some-natalie.dev/blog/cosign-disconnected/
  https://edu.chainguard.dev/open-source/sigstore/rekor/an-introduction-to-rekor/
  https://docs.rs/sigstore/latest/sigstore/
  https://blog.sigstore.dev/rekor-v2-ga/

FACTS CONFIRMED:
  - cosign --offline=true: the Rekor inclusion proof is STAPLED into the Sigstore bundle at signing time.
    Offline verification uses the embedded proof + a local copy of Sigstore's trusted root (downloaded
    once via `cosign initialize`). No live network access required at verify time.
  - The .sigstore bundle protobuf format bundles: artifact signature, Fulcio certificate, Rekor
    inclusion proof, and signed timestamps. All verifiable offline.
  - sigstore Rust crate (docs.rs/sigstore): version 0.14.0. API described as "still experimental" with
    "may change at any time" caveat. 40.23% documented. Provides cosign verification capabilities
    including transparency log retrieval and inclusion proof verification.
  - Rekor v2 is GA as of 2025, with a 2026 instance planned per the sigstore blog.

WHAT THIS MEANS FOR ELAI:
  I-18 already plans "opt-in Rekor push: the offline blake3-Merkle default ships without it" and
  explicitly defers the --rekor flag to Tier-2. The mechanism described in the report (submit Merkle
  root, staple the offline-verifiable inclusion proof) IS precisely what the I-18 Tier-2 Rekor path
  would implement. The sigstore-rs crate at 0.14.0 is experimental and would require anti-slopsquat
  verification before import (I-18 already mandates this).
  The only additive insight: the report may be suggesting submitting the RRA BUNDLE'S MERKLE ROOT to
  Rekor rather than just the full RRA, which would be cheaper and produce a compact external anchor.
  That is a design detail within the I-18 Tier-2 scope, not a new feature.

NEW vs ALREADY-PLANNED: ALREADY-PLANNED (I-18 Tier-2 Rekor path)
  Fully covered by I-18's opt-in Rekor push. The stapled offline proof mechanism is the standard
  cosign workflow that I-18 would implement. Not a new addition.

---

## 4. RFC 3161 TSA (Trusted Timestamp)

VERDICT: VERIFIED (real standard, real free services, but with caveats)
URLS:
  https://freetsa.org/index_en.php
  https://knowledge.digicert.com/general-information/rfc3161-compliant-time-stamp-authority-server
  https://github.com/sigstore/timestamp-authority
  https://en.wikipedia.org/wiki/Trusted_timestamping

FACTS CONFIRMED:
  - freetsa.org: real, free, RFC 3161-compliant. Certificate updated 2026-03-16, valid until 2040.
    No SLA. Community-operated. No uptime guarantees published.
  - DigiCert: http://timestamp.digicert.com is a real RFC 3161 endpoint. Appears free for timestamping
    (not documented as paid-only), commonly used for Authenticode signing. No explicit SLA found for
    the timestamping endpoint specifically.
  - Sigstore also ships its own RFC 3161 timestamp-authority (github.com/sigstore/timestamp-authority)
    intended as a trust anchor within the Sigstore ecosystem.
  - RFC 3161 TSA gives: independent third-party proof that a specific blob hash existed at time T,
    bound by the TSA's key (not your local clock). The TSA's timestamp token is a signed DER structure.
  - Build complexity: an RFC 3161 request is a small HTTP POST with the hash of your blob; parsing the
    response is ~50-100 lines of ASN.1 / DER in Rust (via der or rasn crates). Not large.

WHAT THIS MEANS FOR ELAI:
  Our RRA bundle already has a local timestamp from the OS clock, which is self-signed and
  manipulation-susceptible during the run window. RFC 3161 adds an INDEPENDENT external timestamp
  signed by a third party, proving the bundle existed before time T without trusting our clock.
  I-18 and G-8 do not plan RFC 3161. Sigstore's Rekor also provides a signed timestamp via its
  transparency log entry (the LogEntry has a Integrated Time field, signed by Rekor's key) — so if
  the I-18 Tier-2 Rekor push lands, you get a Rekor-signed timestamp "for free" as part of the
  inclusion proof, without needing RFC 3161 separately.
  Pure RFC 3161 only adds value if you want a timestamp from a CA-chain-rooted TSA (X.509 trust)
  rather than just the Rekor log's self-signed timestamp. For ELAI's use case (developer tool,
  not a legal/notarial requirement), Rekor's timestamp is adequate and already covered.

NEW vs ALREADY-PLANNED: RIDER-ON-EXISTING (covered by Rekor's integrated timestamp in I-18 Tier-2)
  If Rekor push lands (I-18 Tier-2), you get a TSA-like signed timestamp via Rekor's LogEntry for
  free. A dedicated RFC 3161 call would add X.509 CA trust-chain provenance on the timestamp, which
  is only needed for legal/compliance contexts (e.g. Authenticode-style NDA chains). Not worth a
  separate build for ELAI's current scope.

---

## 5. in-toto + SLSA v1.0 Provenance

VERDICT: VERIFIED (standard is real, schema maps well to RRA, offline verification is supported)
URLS:
  https://slsa.dev/spec/v1.2/build-provenance
  https://slsa.dev/blog/2023/05/in-toto-and-slsa
  https://github.com/slsa-framework/slsa-verifier
  https://secure-pipelines.com/ci-cd-security/artifact-provenance-attestations-slsa-in-toto/

FACTS CONFIRMED:
  - SLSA v1.0 predicate schema (now at v1.2, backwards compatible): buildDefinition contains buildType,
    externalParameters, internalParameters, resolvedDependencies. runDetails contains builder (id),
    metadata (startedOn, finishedOn), byproducts.
  - in-toto Statement wraps any predicate: { _type, subject (name + digest map), predicateType, predicate }.
    Subject digest can be any hash algorithm key ("sha256", "blake3" if agreed by verifier).
  - Mapping to RRA: builder.id = ELAI version + model_pin; externalParameters = task_fixture + ELAI_SEED;
    resolvedDependencies = Cargo.lock blake3 hash (DEPS layer from I-18's 5-layer stack hash);
    metadata.startedOn/finishedOn = run timestamps; byproducts = SSB event count + verdict hashes.
    This mapping is clean and natural.
  - slsa-verifier: verifies SLSA provenance from a .sigstore bundle offline (verifies signatures,
    builder ID, source repo). cosign verify --attestation with --type slsa_provenance also works.
  - cosign can consume in-toto attestations with --offline flag (same stapled-proof mechanism as item 3).

WHAT THIS MEANS FOR ELAI:
  G-8 ALREADY plans to emit a "custom in-toto v1 predicate" with model id, ELAI_SEED, F-gate verdicts,
  and an SLSA-VSA-style verifiedLevels rollup. The report is suggesting we standardize on the official
  SLSA Provenance predicate type instead of a fully custom predicate. The difference: using the official
  predicateType "https://slsa.dev/provenance/v1" means slsa-verifier and cosign can consume the
  attestation without custom tooling; a fully custom predicate requires a custom verifier.
  Adopting the SLSA predicate schema for G-8's attestation is a design refinement within G-8's scope —
  use the standard predicate type and map ELAI's fields into it, rather than defining a bespoke format.
  This makes the attestation interoperable with slsa-verifier out of the box.

NEW vs ALREADY-PLANNED: RIDER-ON-EXISTING (G-8)
  G-8 already plans in-toto Statement + SLSA-VSA-style predicate. The additive suggestion is: conform
  to the official SLSA Provenance v1 predicateType rather than a custom one, so slsa-verifier consumes
  it natively. This is a one-field change (predicateType string + field renaming per the v1.0 spec)
  within G-8's scope. Worth noting as a G-8 design refinement, not a new build.

---

## 6. buf breaking vs serde deny_unknown_fields + #[non_exhaustive] + snapshot test

VERDICT: VERIFIED (honest read — serde approach is meaningfully weaker but adequate for our case)
URLS:
  https://github.com/bufbuild/buf
  https://github.com/serde-rs/serde/issues/2634
  https://github.com/serde-rs/serde/issues/2611
  https://protobuf.dev/programming-guides/proto3/

FACTS CONFIRMED:
  - buf breaking: detects backward-compatibility violations at the Protobuf schema level — field
    number changes, type changes, label changes (required/optional/repeated). Operates on .proto
    files with FILE/PACKAGE/WIRE/WIRE_JSON compatibility modes. Very precise because Protobuf has
    a well-defined wire format spec.
  - serde deny_unknown_fields: rejects deserialization when unknown fields are present. This catches
    ADDITIVE schema changes (a new field from a newer writer causes old readers to fail). The opposite
    of forward-compat. Known issues: does not compose cleanly with skip, tagged enums, flatten.
    Multiple open GitHub issues (2294, 2121, 1864) about edge-case failures.
  - #[non_exhaustive]: prevents external crates from exhaustively pattern-matching enums/structs, but
    provides ZERO wire-format guarantees. It is a compile-time semver tool, not a serialization tool.
  - Snapshot test (insta crate pattern): serialize a struct to JSON/TOML, commit the snapshot, assert
    it does not change across builds. This catches ACCIDENTAL schema changes (a field rename, type
    change). Not a formal compatibility check — it only catches changes relative to the snapshot, not
    whether a change is backward-compatible.
  - Combined (deny_unknown_fields + snapshot test): catches unknown fields on read AND accidental
    schema drift at build time. Does NOT catch: reordering of fields in formats where order matters,
    type widening/narrowing, semantic changes that preserve shape, partial-schema forward reads.

  vs our CURRENT golden lock (action_type_lock.rs):
  Our existing test is a string-match golden lock on the action_type enum discriminants. It catches
  additions/removals/renames of action types. It does NOT check field-level schema of individual
  payload structs.
  
  vs buf breaking (if we were on Protobuf):
  buf breaking would give field-level formal wire-compat checking. We are NOT migrating to Protobuf.
  The report is correct that buf is strictly superior IF on Protobuf, and correctly identifies that
  the realistic alternative for a serde/JSON/TOML codebase is deny_unknown_fields + snapshot.

HONEST READ:
  serde deny_unknown_fields + snapshot test is a meaningful IMPROVEMENT over our current golden lock
  (which only checks action_type names, not payload field shapes). Adding deny_unknown_fields to SSB
  payload structs + insta snapshots for each SsbRow variant's serialized form would catch payload
  schema drift that the current test misses. It is NOT as strong as buf breaking but is the correct
  tool for a serde codebase. This is a real gap in our current golden-lock coverage.

NEW vs ALREADY-PLANNED: NEW (payload-level schema coverage gap not addressed in I-18 or action_type_lock)
  I-18 extends the SSB schema (adds chain_prev_hash field) and mandates the golden lock bump. But
  neither I-18 nor the existing action_type_lock.rs test covers payload field-level schema. Adding
  deny_unknown_fields + insta snapshot on payload structs is genuinely new and addresses a real gap.
  It is a small addition (one attribute per payload struct + one snapshot per SSB action type variant)
  but it closes the field-level drift hole.

---

## Summary Table

| # | Claim | Primary Source | Status | NEW / ALREADY / RIDER |
|---|-------|---------------|--------|----------------------|
| 1 | RFC 6962 O(log n) inclusion + batched checkpoint pattern | datatracker.ietf.org/rfc6962, rfc-editor.org/rfc9162 | VERIFIED | RIDER-ON-EXISTING (I-18 Tier-2) |
| 2 | PunkGo arXiv:2602.20214 exists, Rust, writer isolation, 3 stars | arxiv.org/abs/2602.20214, github.com/PunkGo/punkgo-kernel | VERIFIED | ALREADY-PLANNED (SSB single-writer is structurally equivalent; gap = contract test only) |
| 3 | Rekor stapled offline inclusion proof (--offline) | some-natalie.dev/cosign-disconnected, docs.sigstore.dev | VERIFIED | ALREADY-PLANNED (I-18 Tier-2 Rekor path) |
| 4 | RFC 3161 TSA freetsa.org / DigiCert free service | freetsa.org, knowledge.digicert.com | VERIFIED (no SLA on freetsa) | RIDER-ON-EXISTING (Rekor gives equivalent timestamp; separate RFC 3161 only for X.509 CA trust) |
| 5 | in-toto + SLSA v1.0 predicate maps to RRA; offline via cosign | slsa.dev/spec/v1.2, slsa-verifier GitHub | VERIFIED | RIDER-ON-EXISTING (G-8 already plans custom in-toto; rider = adopt standard predicateType) |
| 6 | serde deny_unknown_fields + snapshot test as buf alternative | github.com/serde-rs/serde issues, bufbuild/buf | VERIFIED | NEW (payload-level schema drift not covered by existing golden lock or I-18) |

---

## Writer Isolation: I-1 vs Journal-Specific

I-1 addresses EXECUTION isolation: the generator (LLM process) cannot call any sink (filesystem, shell,
network) directly. The Supervisor is the sole executor.

PunkGo's isolation is narrower: agents cannot write to the AUDIT LOG directly; only the kernel can.

For ELAI's SSB: the single-writer constraint is already structural (ssb.rs is the only write path;
the WAL is not a public multi-writer API). PunkGo's design formalizes this as an explicit security
invariant with a contract test and a capability check. The GAP for ELAI is: we have the structural
property but no contract test asserting it and no formal statement of the invariant.

Verdict: I-1 covers the broader execution-isolation story. The journal-specific writer-isolation
invariant is an additional contract test, not a new architectural component. It could be a small
addition to I-18 or a standalone patch: one contract test in crates/contracts/tests/ asserting
"no code path outside ssb.rs::write_payload can insert an SsbRow."

---

## Single Highest-Value Genuinely New Addition

Item 6 (serde deny_unknown_fields + insta snapshot on payload structs) is the only GENUINELY NEW
finding. All others are either already planned (I-18/G-8/G-2) or riders that are valuable only if
the parent item is already doing work nearby.

Why item 6 is the highest-value new addition: our current golden lock checks action_type NAMES
but not payload FIELD SHAPES. I-18 adds chain_prev_hash to every SsbRow — if that field is
accidentally dropped in a future schema migration, the golden lock (action_type string match) would
not catch it. deny_unknown_fields + insta snapshot on payload structs would catch it at compile
time. Cost: one attribute per struct + one snapshot file per action type variant. Zero new deps
(insta is already in common Rust test stacks; verify before import). High yield, low cost.

---

## Verification provenance

All 6 items grounded from primary sources (IETF RFCs, arXiv paper, GitHub repos, official docs).
No claim taken from training memory alone. The exact multiplier in the "850x slower per-event Merkle"
claim is unverifiable from the RFC text (principle confirmed, number not); labeled accordingly.
freetsa.org has no SLA — noted as community-operated, no uptime guarantee.
sigstore-rs at 0.14.0 is experimental per its own README.
