# ELAI: agent-harness research

> [!WARNING]
> **Restart planned following a development hiatus.**
> I intend to resume work on the full ELAI project. This repository currently
> contains unfinished research code. Historical implementation, security and
> benchmark claims have not been revalidated. Do not expect a working product.
> This status update announces that commitment; it does not establish new
> functionality, completed fixes or a release date.

ELAI explored a local-first, model-agnostic agent harness: models propose actions,
while software owns planning, state, permissions, execution, evidence,
verification and the decision that work is complete. That was the design goal;
it is not a claim that every path enforced it correctly.

I used it to try too many ideas headlessly, not as software I relied on for actual
work. Retrieval, orchestration, verification, sandboxing, memory, model routing,
self-improvement and UI experiments kept accumulating. It became overengineered,
and I stopped development at that time. I now intend to return to the project. The lesson is simple: **build software you want to use that
solves your problems, not all the problems you know about.**

## Five ideas worth investigating

- **Evidence outside the generating agent:** judge work through independently captured artifacts.
- **Fail-closed verification:** reject missing evidence instead of treating it as a skipped check.
- **Journals and replay:** preserve what happened and explore reproducible execution.
- **Bounded workers:** keep task state, permissions and completion decisions in the harness.
- **Code retrieval:** combine keyword/vector search, dependency graphs and repository maps within a context budget.

These are extraction candidates and design references. The [salvage map](SALVAGE-MAP.md)
links to source code, integration points and concrete limitations.

## Code, experiments and plans

| Material | What you can conclude |
| --- | --- |
| [Rust, Python and Tauri source](.elai_cc/) | Implementation exists; current buildability and end-to-end behavior are unverified. |
| [Benchmarks and recorded results](BENCHMARKS.md) | Harnesses, fixtures and historical result artifacts are included; their scope and missing inputs matter. |
| [659-entry feature catalog](docs/FEATURE-CATALOG.md) | These are plan units and experiments, not 659 working features. |

The archive preserves **883 real historical commits**, with privacy edits to their
contents and identities, followed by archive-preparation commits. Dates and parent
relationships were retained; see the [history review](HISTORY-REVIEW.md).

## Start here

- [Postmortem](POSTMORTEM.md) — why the project expanded and what I would do differently.
- [Salvage map](SALVAGE-MAP.md) — concrete places to start reading or extracting.
- [Benchmark guide](BENCHMARKS.md) — what was committed and what the results actually cover.
- [Conductor](CONDUCTOR.md) — the external automation that coordinated ELAI's development, with a guide to the selected source.
- [Detailed research overview](RESEARCH-OVERVIEW.md) — the former long README, including the full catalog.
- [Recovered documentation](docs/RECOVERED-DOCUMENTS.md) — dashboard designs, mockups, research sweeps, experiment reports and workflow guides.
- [Systems specification](docs/thefinalglance/ELAI-SPEC-OVERVIEW.md) — the intended architecture.
- [Archive guide](ARCHIVE.md) and [publication review](PUBLICATION-REVIEW.md) — selection, privacy edits and limitations.

Older files also call the project **LIA**. Read, fork or extract whatever is useful,
expecting to do your own validation. Original code and documents use [MIT](LICENSE);
[third-party notices](THIRD_PARTY_NOTICES.md) cover retained upstream material.
I plan to resume development and maintenance across the project. Release timing
and support availability are not yet established.

[Architecture preview](docs/assets/social-preview.png) · [Editable diagram](docs/assets/social-preview.svg)
