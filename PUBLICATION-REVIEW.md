# Publication review — abandoned ELAI archive

Prepared 2026-09-06 for the owner's review. Publication is a separate owner action.
The [README](README.md) describes the project as an abandoned, unsupported research
playground. Nothing in this review establishes that ELAI builds, runs, is secure,
or delivers its planned features.

## Contents

The prepared snapshot contains 6,459 selected files plus
`ARCHIVE-MANIFEST.sha256`, approximately 128 MiB in total. It includes experimental
source and 2,727 Markdown/text documents across the original planning, research,
historical agent instructions and memory-note directories. Downloaded repositories,
private conversations and runtime artifacts are outside that selection.

The [feature catalog](docs/FEATURE-CATALOG.md) contains 659 phrases across 17
families: foundational PHASE and B–Q. Each phrase names a plan unit that may bundle
multiple behaviors. It is a navigation index, not a claim of 659 working features.
The catalog reconciles 613 original plan files with the exhaustive source: 612
matched units and 47 exhaustive-only units, with no unmatched original feature
plans. All original PHASE/B–Q plan files considered by that reconciliation remain
included. Supporting/index headings and aliases are explained in the catalog.

Original ELAI code and documents use [MIT](LICENSE), with “ELAI contributors” in
the copyright notice. Upstream material retains its own terms and attribution;
see [third-party notices](THIRD_PARTY_NOTICES.md).

Suggested hosting description:

> ABANDONED EXPERIMENT: an overengineered agent research playground, with code and original plans. Do not download expecting a working product. Unsupported; no maintenance planned.

## Preparation evidence

- Known personal names, handles, emails, account IDs, machine paths and private
  endpoints were removed, generalized or excluded from the selected text and
  filenames. Contextual review also removed private collaboration, conversation,
  deployment and personal-project documents. Public citations and legal credits
  retain their attribution.
- The snapshot scan found zero unresolved known-identity, credential-pattern or
  excluded-artifact matches. It reviewed 73 synthetic secret-pattern occurrences
  through 53 exact path/type/value-hash allowances. These are deliberate detector
  fixtures and header patterns, not a blanket exclusion for entire files. Identity
  findings cannot be allowlisted. Private search terms and original-value reports
  stay outside the snapshot.
- Ten binary fixtures remain: two tiny tar test archives, three synthetic
  perception images, four transparent bundle icons and a tiny PNG fixture stored
  as `.bin`. Their contents or imagery were inspected separately. The tar path
  traversal fixture is deliberately adversarial test data; it was inspected
  without extraction. A small readable formula PDF is also synthetic fixture data.
- Catalog regeneration passed `--check`. Local targets and numeric source-line
  references in the new README, catalog and guides were checked. Historical links,
  external URLs and old claimed evidence were not exhaustively validated.
- The exported file set matched the selected Git index. The checksum manifest
  covers every exported file except itself; every listed hash was verified.
- Export and audit behavior was exercised in disposable repositories: staged
  bytes only, no untracked/dirty additions or Git metadata, destination refusal,
  symlink refusal, checksums, identity/artifact detection and exact allowlist
  behavior. Regression coverage includes an existing source manifest and a
  destination whose parent symlink resolves inside the source repository.

## Deliberate omissions and limits

The [archive guide](ARCHIVE.md) describes selection and privacy edits. Local
originals were retained privately. Exclusions include runtime databases, indexes,
agent sessions, downloaded upstream source trees, copied prompt dumps, raw paper
PDFs/research images, tool binaries and private infrastructure notes.

Four HaluEval/MMLU inputs, 20 adapted sycophancy fixtures and their generator, and
an unused default Tauri logo were omitted because their provenance or reuse terms
were not sufficiently established for this snapshot. Their records remain in
the notices. The 80 synthetic sycophancy fixtures and Apache-covered TruthfulQA
material remain. Old benchmark checks needing omitted inputs cannot be reproduced
from this archive alone.

Review was bounded. Pattern searches cannot guarantee that all personal context
or secrets have been identified. The exact starting Claw import revision remains
unknown; the current upstream MIT notice is retained. There is no complete
file-by-file provenance or transitive dependency license audit.

No ELAI application build, product test suite, model inference or historical
automation was run for this preparation. Changes to comments, fixture identities
and private endpoint defaults were reviewed as privacy edits. Endpoint placeholders
are intentionally unusable as the former services. Sanitization also invalidates
historical byte hashes, signatures and line offsets. Use the new manifest for
this snapshot, not old receipts.

## Owner review and publication

Review the separate `elai-public-history-review` repository. This edition retains
883 sanitized main-history commits, followed by archive-preparation commits.
Its original dates and parent relationships were verified; no replacement commits
were invented to inflate the count. Author/committer identities were anonymized,
private content was filtered and original signatures were stripped. The
[history review](HISTORY-REVIEW.md) describes these changes and emptied commits.

**Keep the original development repository private. Publish the filtered history
review copy.** The older `elai-public-review` folder remains available as the
separate single-commit edition. Neither copy has a remote, and nothing was pushed.
The publishing account's own identity will remain visible on the hosting service.

Read the README, catalog, history review and exclusions before publishing.
Changes to the latest snapshot require regenerating its manifest and repeating
relevant checks; changing privacy policy also requires reviewing historical
versions. No remote creation or repository visibility change is part of this work.

Archive-only checks, from the history-review root:

```sh
python3 scripts/archive/build_feature_catalog.py --check
python3 scripts/archive/test_export_public_snapshot.py
python3 scripts/archive/test_audit_public_history.py
python3 scripts/archive/audit_public_snapshot.py . --terms-file /path/outside/repo/private-terms.json
python3 scripts/archive/audit_public_history.py . --terms-file /path/outside/repo/private-terms.json
sha256sum --check ARCHIVE-MANIFEST.sha256
```

The optional private-terms file is a JSON array of strings and must remain outside
the public repository. Without it, the audits cannot check owner-specific
identifiers. The snapshot exporter produces files without Git history; it must
not be used when the intended output needs the preserved commit timeline.
