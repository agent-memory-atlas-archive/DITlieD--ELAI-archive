PHASE 3 DECISIONS

WHAT

Expose one dry-run/report path for all five requested suites, make future harness adapters explicit, and avoid cross-suite score aggregation until comparability is proven.

KEY DECISIONS

- `--all` always includes cloneable suites plus UnderSpecBench paper-only, so coverage gaps are visible.
- `--dry-run` constructs deterministic command/report metadata and writes no adapter process output.
- `--execute` is mandatory for any adapter subprocess; report paths, workspaces, logs, and normalized artifacts are constrained to `.elai_cc/benchmarks/harness_trust/`.
- The existing project benchmark registry stays unchanged because this integration has external-suite lifecycle statuses and no comparable umbrella score.
- README documentation is the user-facing plug-in contract; no native harness CLI or secret is installed by the setup.

EXPECTED TEST OUTCOME

- A fixed dry-run reports five suites, one `dry_run_ready` when a command is supplied, three `adapter_required`, and one `paper_only`.
- Aggregate score remains null and disabled.
- A safe adapter smoke can write one JSONL evidence row, which the runner normalizes and records without invoking a model.

UNKNOWNS

- Real suite adapter commands are harness-specific and require user-provided executable paths, model IDs, credentials, and possibly Docker.
- Score comparability across upstream suites is not established by this setup and must not be inferred from numeric fields alone.
