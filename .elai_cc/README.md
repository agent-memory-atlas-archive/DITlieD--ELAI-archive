# ELAI core (`.elai_cc/`)

This is the project root for the ELAI deterministic development engine. The
top-level repo README at `../README.md` covers product positioning; this file
documents operational concerns the binaries surface.

## Tamper-Recovery (D-8 Archive Verify)

The reasoning-template ArchiveChain (`.elai_cc/archive/reasoning-templates.jsonl`)
is the append-only ledger of every promoted, frozen, and retired
ReasoningTemplate the orchestrator has ever produced. Each entry carries a
blake3 `self_hash` over `(template_body, parent_hash)` plus a `parent_hash`
reference to its predecessor. The chain is intentionally line-diffable in
git review (DGM §5 archive practice) so any out-of-band edit is visible at
PR time.

### Detecting tampering

Two equivalent detection paths land the same diagnosis:

1. **Developer side — `elai archive verify`**
   ```
   elai archive verify --path .elai_cc/archive/reasoning-templates.jsonl
   elai archive verify --json --fail-fast    # CI mode
   ```
   Exit `0` = clean chain. Exit `1` = at least one of:
   - `HashMismatch` — a `template_body` was rewritten without recomputing
     `self_hash` (canonical lobotomy attack).
   - `BrokenParent` — an entry's `parent_hash` references a hash that was
     never appended earlier (forged ancestry or chain-truncation attack).
   - `DuplicateHash` — two entries share a `self_hash` (accidental
     double-append, or a cryptographically improbable collision attempt).
   - `MalformedJson` — a JSONL line failed to deserialise; the verify
     driver uses the lenient loader so subsequent lines continue to be
     walked.

   The `--json` flag prints the typed `ArchiveVerifyReport` (jq-parseable
   single object). The `--fail-fast` flag short-circuits on the first
   error — recommended for CI; the collect-all default is recommended for
   local debugging so you see every problem in one pass.

2. **CI side — `.github/workflows/archive-verify.yml`**
   Runs `cargo build --release --bin elai` then invokes
   `elai archive verify --fail-fast` against the in-repo archive on every
   push and pull_request to any branch. The workflow skips itself if the
   archive file is absent so PRs that don't touch the archive aren't
   blocked by a missing fixture.

Every invocation — pass or fail — also writes a single
`SsbActionType::ArchiveVerifyResult` row to the project's session SSB DB
so dashboards can render a verify-history timeline without re-walking the
JSONL.

### Recovery procedure (verify failed)

If `elai archive verify` reports a failure, **do not edit the JSONL by
hand**. The repair path:

1. **Identify the tampered range.** Run the verify driver in collect-all
   mode (no `--fail-fast`) and note every `entry_index`. The lowest
   index is the earliest divergence from a trusted state; everything
   from there forward is suspect because hash-chain semantics make
   later entries depend on earlier ones.

2. **Cross-check against the RAG / sidecar snapshots.** The audit-log
   retention policy keeps a 30-day window of SSB rows for every
   `ArchiveEntryAdded` and `ArchiveEntryRetired` event. Each row carries
   the `template_id`, `content_hash`, and `parent_hash` at the moment of
   the append. Reconstruct the trusted state by walking the SSB log
   forward from the most recent verified-clean checkpoint.

3. **Rebuild the chain from the trusted snapshot.** Append-only
   semantics mean repair is *not* in-place editing — it is a NEW chain
   that replays the trusted history forward. The old (tampered) chain
   is preserved verbatim under `archive/quarantine/<timestamp>.jsonl`
   so forensic analysis can compare the divergence.

4. **Re-run `elai archive verify` on the rebuilt chain.** Exit `0` is
   the gate; the SSB row for that invocation is the audit trail.

### Trust positioning

The chain rationale is supported by two pieces of literature (neither
specifies the implementation; see
`workflow/reasoning/D-8-decisions.md` DECISION-5):

* arXiv:2507.05619 (T1 cryptographic-checksum recommendation) frames the
  threat model — agents rewriting their own audit history to suppress
  reward-hacking evidence.
* DGM §5 (archive practice) frames the design pattern — append-only
  lineage exposes diff so the §H hallucination-marker-removal incident
  could not have gone undetected.

Future hardening tracks (orthogonal to D-8):
* **SLSA Build L3** — generate a provenance attestation for every
  release that pins the source commit + build environment, so
  consumers can verify the binary was built from the audited chain.
  Status: deferred to a release-engineering wave.
* **Sigstore signing** — cosign-sign the JSONL on every committed
  promotion so the chain is verifiable against a public transparency
  log even outside ELAI's session SSB. Status: deferred to D-9 (Locked
  Keys Auditor) integration.
