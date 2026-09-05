# Retained development history

This edition preserves the real main-branch development timeline. It contains
883 historical commits, followed by archive-preparation commits.
No backdated commits or replacement activity were invented to inflate that count.

## What remains

The 883 original commit nodes retain their author/committer dates and parent
relationships, including two merges. Their descriptions and changes remain where
compatible with the publication policy. Commits emptied by removing excluded
material remain as real historical nodes; an empty sanitized diff is not evidence
that the original commit did no work. Eleven previously nonempty commits became
empty after filtering; two were already empty. Both merge commits remain nonempty.

This is a privacy-filtered history. Author and committer identities are normalized
to ELAI contributors, and personal identifiers and attribution trailers were
removed from commit messages. All commit hashes changed. Nine old commit signatures
were stripped because they could not authenticate the edited objects. Public
copyright notices, licenses and citations in retained files remain attributed.

The original development repository is unchanged by this history preparation.
Only its main ancestry was copied into an isolated repository and rewritten.
Other local branches, remote refs, tags and their additional commits are outside
this edition. The prior single-commit `elai-public-review` edition is still
available separately.

## Historical content selection

The history uses the same reviewed path selection as the snapshot, applied at
every revision. Of 6,889 path names found in main history, 433 outside the reviewed
selection are omitted. This includes private/runtime material and unreviewed
historical-only files, including some removed source files. It is not a byte-for-byte
or complete historical source backup.

Ten binary fixture paths were omitted from the historical revisions. Their exact
reviewed current versions were restored during archive preparation, alongside
the current README, feature catalog, notices and manifest. This leaves 6,446
selected historical paths before the preparation commit. Unreviewed binary
payloads are not silently treated as scanned text.

Known identity terms were removed throughout retained file versions. Credential
patterns outside the existing exact synthetic-fixture allowances were redacted.
Some isolated historical non-UTF-8 punctuation was converted to UTF-8. These
changes may invalidate old tests, references, hashes and claimed measurements.
No old application version was built or run as part of the review.

## Verification

The preparation compared the original and filtered commit graph, dates and
metadata, in addition to scanning the selected file contents. Before the
archive-preparation commits, the full-history scan covered all 883 commits, 11,510 unique
retained file blobs and 6,446 paths. It found zero unresolved known-identity,
credential-pattern or excluded-artifact findings. The 100 synthetic-pattern
occurrences across unique historical blobs were covered by the existing exact
fixture fingerprints; no new history-wide pattern exemption was added.

The history audit reads complete commit objects, including metadata and messages,
and file blobs reachable from every ref. Its regression tests demonstrate that
it detects removed sensitive text, old commit-message identities and content on
a non-HEAD branch. A shared fixture blob must have an exact allowance for every
historical path, so moving it cannot silently reuse another path's exception.

The final repository has only its main branch, no remote and no original-history
backup refs. Temporary rewrite maps and private scan inputs are kept outside the
review repository. The original source and the older single-commit edition remain
private and intact.

## Limits and publication

Pattern scans and bounded contextual review do not guarantee complete anonymity,
complete provenance or a functional application. Exact dates and technical design
context remain visible. The publishing account's identity is separate from the
contents of this repository. Old claims and benchmark results remain unverified.

Review [PUBLICATION-REVIEW.md](PUBLICATION-REVIEW.md), [ARCHIVE.md](ARCHIVE.md) and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) before publishing. Publish only this
separately filtered review copy if you want the retained timeline. Do not make the
original development repository public.

The rewrite used [git-filter-repo](https://github.com/newren/git-filter-repo/blob/main/Documentation/git-filter-repo.txt)
with empty-commit and degenerate-merge pruning disabled. `audit_public_history.py`
is included for repeatable bounded inspection; owner-specific terms must be supplied
privately outside the repository. The snapshot exporter intentionally omits history.
