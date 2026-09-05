DIGEST: p0-journal
SOURCE: 4f65e2dd-db7b-420a-b53e-55407c6c5eee.txt

TOPIC: Pillar P0 (trusted run journal). Audits ELAI's design (SQLite-WAL single-writer append-only event journal, blake3 hash-chained rows, blake3 sealed bundles, golden-lock schema freeze) vs tamper-evident-log / event-sourcing / agent-auditability SOTA. Verdict frame: KEEP core, HARDEN with named additions.

REPORT VERDICTS:
- Overall: HARDEN — DO NOT REPLACE. SQLite-WAL single-writer is the correct foundation (operationally mature, crash-safe, queryable, at 80K inserts/sec). No alternative embedded store justifies migration.
- 4 concrete weaknesses: (1) hash chain gives sequential integrity but NOT inclusion proofs without full O(n) replay; (2) writer process is inside the trust boundary (compromised agent can forge events pre-commit); (3) schema-freeze is hand-rolled/fragile vs off-the-shelf tooling; (4) sealed bundle has no interoperable attestation layer external verifiers can consume.
- Hash chain vs Merkle: ADOPT batched Merkle checkpointing (continue linear append, periodically compute Merkle root + Ed25519 sign). Naive per-event Merkle insertion ~850x slower; batch is the correct pattern.
- Google Trillian: NOT practical embedded (needs MySQL/MariaDB + Log Server + Log Signer; distributed server infra not embeddable Rust lib).
- Sigstore Rekor: ADOPT as anchor-only (not storage layer for 80K/sec); submit Merkle root, staple offline inclusion proof.
- PunkGo: directly adoptable RFC 6962 checkpointing layer on top of SQLite row stream.
- Storage alternatives: KEEP SQLite — redb/fjall/sled/varvedb/nebari/okaywal either lack tamper-evidence or too immature.
- Schema freeze: buf breaking (if migrate to Protobuf) strictly superior to hand-rolled golden-lock; else serde deny_unknown_fields + snapshot test.
- Provenance standards: ADOPT in-toto+SLSA v1.0, Sigstore-bundle-as-anchor, RFC 3161 TSA; CONSIDER W3C PROV; SKIP C2PA for core journal.
- Writer-process compromise = THE most fundamental gap.

CANDIDATES (name | url-or-id | report's claim | replaces-or-hardens):
RFC 6962 / RFC 9162 Merkle tree | datatracker rfc9162 | CT-style tree gives O(log n) inclusion proofs + O(log n) consistency proofs; structurally superior to O(n) hash chain | UPGRADE ELAI hash chain (batched checkpoint)
Google Trillian | github.com/google/trillian (Apache 2.0, ~3.5K stars) | production backbone of CT/Go-mirror/Sigstore-Rekor; requires MySQL + Log Server + Log Signer; serverless-log variant closer but no native Rust lib | NOT embeddable (rejected)
Sigstore Rekor | github.com/sigstore/rekor (Apache 2.0) | immutable transparency log; offline-verifiable via stapled inclusion proofs, --offline zero network; not a storage backend for 80K/sec | ADOPT as out-of-band anchor (staple inclusion proof into sealed bundle)
PunkGo / punkgo-kernel | arXiv:2602.20214 github PunkGo/punkgo-kernel | Rust "sovereignty kernel" w/ RFC 6962 Merkle log; agents untrusted by design, kernel sole commit point; Ed25519+Merkle+RFC 3161 TSA; companion punkgo-jack hooks Claude Code/Cursor | directly adoptable checkpointing layer + writer-isolation model; CLOSEST existing system to ELAI design goals
"Right to History" sovereignty kernel | arXiv:2602.20214 | formalizes 5 invariants (append-only, completeness, integrity, boundary enforcement, energy conservation) w/ proof sketches; threat model = agent itself is adversary | grounds ELAI threat model
"Auditable Agents" | arXiv:2604.05485 | 5 dimensions of auditability; pre-execution mediation w/ tamper-evident records adds only 8.3ms median; 617 security findings across 6 OSS agent projects | confirms ELAI design practical
"Verifiability-First Agents / OPERA" | arXiv:2512.17259 | Action Attestation Layer (AAL) signs receipts per external action -> Provenance Log; OPERA benchmark | maps to ELAI anti-fabrication check
"Replayable Financial Agents / DFAH" | arXiv:2601.15322 | trajectory determinism >=90% for replay, evidence grounding >=80% for decisions | grounds ELAI reproducible-benchmark claims
"Private/Verifiable/Auditable AI" | arXiv:2509.00085 | ZK proofs + TEEs for verifiable/auditable claims | hardens writer-compromise threat
"Constant-Size Cryptographic Evidence Structures for Regulated AI" | arXiv:2511.17118 | fixed-size evidence tuples compose w/ hash-chain + Merkle; batch verify 10^5 events in seconds; addresses metadata leakage | applies to ELAI event row schema
PROV-AGENT | arXiv:2508.02866 (IEEE eScience 2025) | extends W3C PROV + leverages MCP for end-to-end provenance in agentic workflows | cross-tool/multi-agent provenance interop
"Attestable Audits" | arXiv:2506.23706 | runs benchmark evals inside TEEs (SGX/TrustZone) | verifiable benchmark reproducibility (Priority 3)
in-toto + SLSA v1.0 | slsa.dev/spec/v1.0/provenance | signed provenance attestations w/ DSSE envelope; covers builder ID/invocation/materials/timestamps; offline self-contained JSON | ADOPT — wrap sealed bundle as in-toto Statement, SLSA schema maps to model-pin+seed+verdict
Sigstore bundle (stapled Rekor) | trustification.io | offline-verifiable bundle w/ stapled Merkle inclusion proof; via sigstore-go or sigstore-rust | ADOPT as anchor
C2PA | c2paviewer.com / c2pa.ai | content provenance for media; EU AI Act (Aug 2026) requires labeling generated content | SKIP for journal; maybe for agent-GENERATED artifacts
W3C PROV | no-url (PROV-AGENT extends) | generic provenance graph (entities/activities/agents) | CONSIDER if cross-tool interop needed; heavyweight for solo embedded
RFC 3161 TSA | freetsa.org | trusted-timestamp authority, independent proof-of-existence at time T | ADOPT — anchor bundle to wall-clock w/o trusting local clock (PunkGo does this)
buf breaking (Protobuf) | buf.build/docs/breaking | 4 graduated compat levels WIRE/WIRE_JSON/PACKAGE/FILE; catches field rename/type change/number reuse mechanically; CI GitHub Action | REPLACES ELAI hand-rolled golden-lock (if migrate to Protobuf via prost)
serde #[non_exhaustive] + #[serde(deny_unknown_fields)] + strum snapshot test | no-url | type-system-enforced variant lock vs string-matching | hardens golden-lock if staying serde
redb | github.com/cberner/redb (~3.7K stars) | pure-Rust ACID MVCC copy-on-write B-tree; append-only by API convention only | no gain over SQLite (rejected)
fjall | github.com/fjall-rs/fjall | 100% safe Rust LSM, WAL-backed crash-safe, fastest batch writes; v2.0-3.x active | better batch throughput but needs compaction, no tamper-evidence (rejected)
sled | sled.rs | BW-tree crash-safe but stagnant development | avoid
varvedb | varvedb.org (v0.4.2) | explicit append-only event store on LMDB; 1M+ events/sec batch, sub-us zero-copy rkyv reads; low adoption, not security-audited | interesting throughput, not audited (rejected)
okaywal / walogs / walcraft | docs.rs/walogs | crash-safe append-only WAL primitives, fdatasync per commit, torn-write detection | useful as durability layer under custom index, not a replacement
nebari | github.com/khonsulabs/nebari | append-only B-tree file format, interrupted writes truncated on recovery; most structurally honest, low adoption | rejected (low adoption)
eventus | github.com/tqwewe/eventus | sequential disk-backed event store, not audited | too immature
VAP (Verifiable AI Provenance) | dev.to veritaschain | cross-domain meta-standard spec, no production adoption | reference

ATTACK-THE-DESIGN (report's strongest criticisms of ELAI):
- Hash chain proves INTERNAL consistency not external fidelity to reality; verifying single-event inclusion requires O(n) replay from genesis.
- Writer-process compromise is THE most fundamental gap: a compromised orchestrator can write fabricated events before hash-chaining and the chain still validates. Mitigations (increasing strength): sole-committer isolation (PunkGo, separate process/narrow API), out-of-band hash anchoring (Rekor/git/S3 Object Lock), TEE-backed writer, external hash witness at test-verdict time (different OS user).
- Golden-lock is hand-rolled, lock file can be updated carelessly w/o review, no machine-checkable compatibility semantics (only identity).
- Sealed bundle verifiable only by custom code; no interoperable attestation surface (cosign/sigstore-go/slsa-verifier can't consume).
- WAL failure modes: checkpoint starvation / unbounded WAL growth from long-running readers; WAL unsupported on NFS/CIFS (silent corruption); SQLITE_BUSY from multiple processes.
- 80K inserts/sec only achievable w/ batched transactions, not autocommit-per-event ("verify on your actual hardware").
- EU AI Act Article 12 (Aug 2026) requires lifetime logging for high-risk AI; Articles 15/73 make cryptographic audit trails the rational compliance mechanism.

PRIORITIZED HARDENING (report's ordered recommendation):
- P1 (high impact, low complexity): add RFC 6962 Merkle checkpointing (every ~1000 events or run-end, Merkle root + Ed25519 sign, stored as MerkleCheckpoint event); isolate writer process from agents (separate OS process, narrow IPC, no direct SQLite handle); add RFC 3161 TSA timestamp to bundles (Freetsa.org/DigiCert).
- P2 (medium): migrate to buf breaking for schema freeze (Protobuf, WIRE level min); wrap sealed bundles in in-toto SLSA provenance (run_manifest.json Ed25519-signed); staple Rekor inclusion proof.
- P3 (high assurance, significant complexity): TEE-backed writer (SGX/TrustZone) for public benchmark claims; monitor WAL checkpoint health (PRAGMA wal_checkpoint(RESTART) timing + WAL size alerts).
Recommended sealed_bundle/ structure: events.db + merkle_root.bin + run_manifest.json (in-toto SLSA predicate: model_pin/seed/verdict_hashes/builder_id/times) + manifest.sig (Ed25519) + rfc3161_timestamp.tsr + rekor_bundle.json + blake3_root.hex.

NUMBERS (figure | claimed source):
- Naive per-event Merkle insertion ~850x slower than baseline append (2025 study, informatika ITB makalah).
- PunkGo: sub-1.3ms median action latency, ~400 actions/sec throughput, 448-byte inclusion proofs at 10,000 entries; punkgo-jack 6 stars; punkgo-kernel 3 stars (June 2026, pre-production).
- Auditable Agents: 8.3ms median pre-execution-mediation overhead; 617 security findings across 6 OSS agent projects.
- DFAH: trajectory determinism >=90%, evidence grounding >=80%.
- Constant-Size Evidence: batch verify 10^5 events in seconds on commodity hardware.
- SQLite WAL measured: 10K-50K transactions/sec small writes NVMe; ELAI claims 80K inserts/sec ("at high end, batch to stay there").
- Trillian ~3.5K stars. redb ~3.7K stars.

UNVERIFIED/FLAGGED by report: PunkGo-kernel production readiness (3 stars, pre-production, treat as reference impl not dependency); varvedb "1M+ events/sec" (self-doc, no independent bench, append-only wrapper not audited); Anthropic/OpenAI internal journal designs (no public disclosure as of June 2026); EU AI Act "high-risk" classification of ELAI (legal question, deployment-dependent).
