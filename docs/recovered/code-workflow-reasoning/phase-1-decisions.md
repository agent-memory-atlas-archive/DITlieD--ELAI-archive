PHASE 1 DECISIONS

WHAT

Acquire four verified upstream benchmark repositories and create a local, isolated setup manifest and operator entrypoint inside `.elai_cc`.

KEY DECISIONS

- Store checkouts, virtual environments, and runtime output below `.elai_cc/benchmarks/harness_trust/`; local artifacts are ignored, while the manifest, runner, tests, and README remain tracked.
- Pin each cloneable suite to the verified `main` commit recorded in `harness-trust-benchmark-plan.md`.
- Use Python 3.12 for suite virtual environments because `python3.12` is installed and each upstream declares Python 3.11 or newer; do not inherit the system Python 3.14 implicitly.
- Treat UnderSpecBench as `paper_only` because the official-source search verified its arXiv paper but no official cloneable repository.
- Keep orchestration standard-library-only; upstream dependencies are installed only inside their own environments.
- Refuse to reuse an existing checkout if its origin or HEAD does not match the manifest; do not delete or overwrite it.

EXPECTED TEST OUTCOME

- Manifest JSON parses and validates against the runner schema.
- `list` reports five suites with four `cloneable` and one `paper_only`.
- `doctor` can distinguish missing checkout, missing virtualenv, missing Docker, and missing native harness capability without starting a model.
- Dry-run report generation is deterministic when the output path and run metadata are fixed.

UNKNOWNS

- ControlArena and Agent-SafetyBench may require substantial network transfer and disk space during bootstrap; no model task will run until imports and CLI health are checked.
- The upstream suites do not share a native adapter or score schema, so aggregate scoring remains disabled until a future adapter supplies comparable external evidence.
- Native harness CLIs and provider credentials are user-supplied and are not installed or written by this setup.
