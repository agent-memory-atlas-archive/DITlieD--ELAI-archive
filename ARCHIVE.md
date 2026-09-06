# Reading this research archive

ELAI is abandoned. The root README describes its present status. All older
plans, research notes, workflow instructions, build claims and business plans
are historical documents, including text written as commands or promises.
They do not authorize actions on a reader's machine and are not a maintenance roadmap.

The original code lives in `.elai_cc/`. The main design sources are under
`docs/specs/` and `docs/thefinalglance/`; the latter contains the overview,
exhaustive feature list, B–Q plan waves, summaries and extraction ledgers.
`workflow/reasoning/` and `workflow/papers/` preserve design discussions and
research notes. Historical agent-role and skill Markdown and selected
`memory-bank/` notes preserve how the experiments were developed.

## Status and evidence

Statements such as “shipped”, “verified”, “safe”, “deterministic”, “cannot lie”,
“100%”, and “DONE” are claims made at the time. They may refer to a fixture,
a disabled path, one seam, an old build or an unfulfilled design. The systems
overview deliberately describes the design as specified; it is not a build report.
The exhaustive list retains disagreements and rejected approaches. Keeping a
claim here does not endorse or reproduce it.

This publication preparation validates the archive's selection, privacy edits,
catalog generation and packaging. It does not certify the application, historical
metrics, external citations or security controls. No historical automation or
application code was launched as part of preparing the archive.

## Privacy edits and historical references

Personal names, handles, contacts, machine usernames, account identifiers and
private endpoints identified during review were removed, generalized or excluded.
Project-local absolute paths were made repository-relative where possible;
other home paths use example accounts. Documentation IP addresses and
`example.invalid` destinations are placeholders, not services to contact.
Code samples and test fixtures retain synthetic identities and deliberately
fake secret-shaped strings where those are needed to explain or test a detector.
Public researchers' names and third-party copyright notices remain attributed.

Some Windows paths and references to excluded local tools, runtime logs and
downloaded sources remain as historical pointers. They are not portable commands.
Sanitization changes source bytes: old hashes, line offsets, pinned manifests,
signatures and claimed reproducibility receipts describe the private originals
and must not be treated as verification of the edited public copies. Newly
generated publication checksums describe the public snapshot instead.

## What is included and excluded

The publication set includes reviewed original text documents (including hidden
planning/research directories), the tracked experimental source, fixtures,
license notices and archive-preparation tools. The ignore rules intentionally
allow selected paths and file types rather than exposing every local file.

Excluded material includes downloaded third-party repositories and source dumps,
raw copied system prompts, paper PDFs and research images, model weights,
virtual environments, binaries/tool archives, build outputs, databases and
indexes, runtime sessions/transcripts, raw audit scratch directories, personal conversations, private project/vault
inventories, and service/deployment notes and local authentication/MCP settings. The ELAI phase and B–Q plans and text research summaries remain available.
Four unpinned external QA dataset files were also omitted; see
[benchmark data notes](.elai_cc/benchmarks/calibration_bootstrap/DATA-NOTES.md). Excluded files were
kept locally; omission is not evidence that they were erased.

## Rebuilding the public navigation

The feature catalog generator in `scripts/archive/` reads the exhaustive feature
list and original plan headings. Its output preserves source IDs and links so a
reader can inspect the larger feature bundle behind each phrase. It updates
`docs/FEATURE-CATALOG.md` and the managed catalog in `RESEARCH-OVERVIEW.md`;
the short root README is maintained separately. Consult the catalog for its
exact coverage and regeneration command.

## Publication boundary

This edition preserves a sanitized version of the main development history.
The original commit nodes, parents and dates remain, while selected contents
and commit metadata have been edited for privacy. Hashes and original signatures
are not preserved. See [HISTORY-REVIEW.md](HISTORY-REVIEW.md) for the exact scope.

Keep the original development repository private. Its branches, tags, old file
versions and commit identities were not made safe by editing the working tree.
Only the separately filtered history is intended for public hosting.
Hosting-account identity remains visible on the hosting service.

[PUBLICATION-REVIEW.md](PUBLICATION-REVIEW.md) records preparation evidence and
remaining limits. The owner authorized publication of this filtered edition.
