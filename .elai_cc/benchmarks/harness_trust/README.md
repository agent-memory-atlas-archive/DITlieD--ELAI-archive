# HARNESS TRUST BENCHMARK

This directory contains the local integration surface for four independently maintained agent-safety or harness-evaluation suites, plus a paper-only record for UnderSpecBench.

The upstream source trees, virtual environments, and run artifacts are local-only and ignored. The pinned manifest and umbrella runner are tracked.

## SOURCES

| Suite | Status | Pinned source |
| --- | --- | --- |
| HarnessAudit | cloneable | `UCSB-AI/HarnessAudit` at `6317162590aeeb1c8dde32b880ac199933343e4a` |
| ControlArena | cloneable | `UKGovernmentBEIS/control-arena` at `3a378ed00eb1f6b8e893e39215d749688126c9cb` |
| Agent-SafetyBench | cloneable | `thu-coai/Agent-SafetyBench` at `74feea8de601b3a1449a93fcf70017fe61556f73` |
| OpenBench | cloneable | `minghinmatthewlam/openbench` at `9364ebaf6279bb533cac073fd2e67f4b7d67d8d9` |
| UnderSpecBench | paper-only | `arXiv:2607.02294`, no verified official repository URL |

The complete URLs, licenses, install metadata, and adapter notes are in `suites.json`.

## REGISTRY BOUNDARY

The existing `.elai_cc/benchmarks/registry.json` remains unchanged. Its rows describe in-process `elai bench` measurements and their project status artifacts, while this integration preserves external suite status, raw evidence paths, and a paper-only state without a comparable aggregate score.

## SETUP

List the suites without network access:

```bash
python3.12 .elai_cc/scripts/trustbench.py list
```

Inspect local capabilities and checkout state:

```bash
python3.12 .elai_cc/scripts/trustbench.py doctor
```

Clone the four verified repositories and install each repository-declared dependency set in its own Python 3.12 virtual environment:

```bash
python3.12 .elai_cc/scripts/trustbench.py bootstrap --all --python python3.12
```

Bootstrap never installs Docker, native harness CLIs, provider credentials, or model weights. ControlArena settings that use sandboxing still need Docker, and every real evaluation needs the upstream suite’s required credentials and harness configuration.

## DRY RUN

Construct one report for all five requested suites without starting a subprocess, model, native harness, or Docker container:

```bash
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/harness \
  --model fixed-model-id \
  --seed 7 \
  --all \
  --dry-run
```

The report is written below `benchmarks/harness_trust/results/dry-run/report.json`. It includes `adapter_required` for suites that do not yet have a command adapter and `paper_only` for UnderSpecBench. No score is inferred from either status.

## ADAPTER CONTRACT

An adapter is supplied with `--adapter-command SUITE=COMMAND`. The command is parsed without a shell and may use these substitutions:

- `{harness}`: the harness executable or path supplied by the caller
- `{model}`: the fixed model identifier
- `{seed}`: the deterministic trial seed
- `{suite}`: the suite ID
- `{suite_root}`: the pinned upstream checkout
- `{workspace}`: the isolated per-suite workspace
- `{output}` or `{artifact}`: the JSONL file the adapter must write

Example adapter shape:

```bash
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/harness \
  --model fixed-model-id \
  --seed 7 \
  --suite openbench \
  --adapter-command 'openbench=python path/to/adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --execute
```

The adapter must write at least one JSON object per line to `{output}`. A numeric `score` is preserved as suite data, but the umbrella report does not aggregate it because the upstream suites do not currently share a comparable score definition.

Each executed command receives these environment variables as well:

- `HARNESS_TRUST_SUITE`
- `HARNESS_TRUST_HARNESS`
- `HARNESS_TRUST_MODEL`
- `HARNESS_TRUST_SEED`
- `HARNESS_TRUST_WORKSPACE`
- `HARNESS_TRUST_OUTPUT`

The runner prepends the selected suite’s virtualenv to `PATH`, captures stdout and stderr, rejects malformed or empty JSONL, and records nonzero exits as `failed`.

## STATUS MODEL

- `cloneable`: official source and pinned ref are recorded
- `paper_only`: a source paper is available, but no official cloneable code was verified
- `adapter_required`: the suite is prepared but no harness-specific command was supplied
- `capability_missing`: checkout or isolated environment is not ready
- `dry_run_ready`: a command was constructed but not executed
- `completed`: the adapter produced valid JSONL evidence
- `failed`: the adapter, timeout, or evidence validation failed

## SAFETY BOUNDARIES

- The bootstrapper refuses to reuse a checkout with a mismatched remote, commit, or dirty state.
- All runner-created workspaces, logs, normalized artifacts, and reports stay under `.elai_cc/benchmarks/harness_trust/`.
- Absolute output paths outside `.elai_cc` are rejected.
- Commands are executed without `shell=True`.
- API keys and secret values are never written by this setup. Secret-looking command arguments are redacted in reports.
- `--execute` is required for any adapter subprocess. Setup and `--dry-run` do not invoke a model or benchmark task.
- UnderSpecBench is not executed from an unverified fork.
