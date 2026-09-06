# ELAI postmortem

ELAI was an ambitious, overengineered experiment that I abandoned. I am publishing
the code, plans and research because some of the ideas may still be useful.
This is a short account of the project, not a claim that it was a finished product.

## What I was exploring

The central idea was a model-agnostic harness. The model could propose work while
software handled state, permissions, execution, evidence and verification.
That idea spread into code retrieval, agent orchestration, context management,
local models, self-improvement, automation and several desktop surfaces. The
[detailed overview](RESEARCH-OVERVIEW.md) preserves the full ambition.

## Why it kept expanding

ELAI became a place to try different ideas. Each problem suggested another
subsystem, another abstraction or another plan. The archive shows how much that
scope grew; the size of the plan corpus does not show how useful the whole thing was.

I was testing things headlessly. I was not using ELAI to get my actual work done,
and polishing the UI/UX was not a priority. There was no daily dependence on the
software forcing me to keep one useful path simple. Eventually I stopped working
on it, and I do not plan to continue.

## What existed, and what was only a claim

There is actual Rust, Python and Tauri code, along with test fixtures, benchmark
harnesses and recorded experimental results. The [benchmark guide](BENCHMARKS.md)
distinguishes saved measurements from simulations, fixtures and plans. Some
experiments also recorded negative results; keeping those is part of the point.

That evidence is narrower than “ELAI worked.” No application build or historical
benchmark was rerun for this archive. Some paths were incomplete, optional or
disabled, and some benchmark inputs could not be included. The [salvage map](SALVAGE-MAP.md)
points out specific limitations, including replay code that validates a request
without providing full deterministic model replay.

The [feature catalog](docs/FEATURE-CATALOG.md) is a map of design intentions.
A plan marked DONE, a component test or a historical score does not establish
that the complete application delivered that feature. There is no credible
single finished-versus-unfinished percentage to put on this archive.

## What I would do differently

**Build software I want to use that solves my problems, not all the problems
I know about.**

For me, the useful change would be to start with a narrow workflow I actually
use and let its problems determine what gets built next. The archive's broad
architecture is a record to learn from, not a checklist for a second attempt.

## Why keep it public?

Someone may want the retrieval code, a verification idea, a record of an experiment
that failed, or an example of how a small premise grew into too much machinery.
The code and original documents are available under [MIT](LICENSE), subject to
[third-party notices](THIRD_PARTY_NOTICES.md). There is no maintenance commitment.

[Back to the introduction](README.md) · [Where to look for reusable pieces](SALVAGE-MAP.md)
