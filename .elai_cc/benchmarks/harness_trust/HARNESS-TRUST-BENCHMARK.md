# HARNESS TRUST BENCHMARK LATER-USE GUIDE

FILE

`.elai_cc/benchmarks/harness_trust/HARNESS-TRUST-BENCHMARK.md`

Run the commands below from:

`/home/user/teikoku/elai`

PURPOSE

This setup gives one local entrypoint for four pinned upstream suites:

- HarnessAudit
- ControlArena
- Agent-SafetyBench
- OpenBench

UnderSpecBench is recorded as `paper_only`. Its paper is available, but no official cloneable repository was verified, so this setup does not execute an unverified fork.

The umbrella runner does not merge upstream implementations or invent one cross-suite score. It keeps each suite’s status and raw evidence separate, then emits one report.

IMPORTANT PATHS

Tracked setup files:

- `.elai_cc/benchmarks/harness_trust/suites.json`, pinned sources and install metadata
- `.elai_cc/benchmarks/harness_trust/README.md`, adapter contract and safety notes
- `.elai_cc/scripts/trustbench.py`, umbrella CLI
- `.elai_cc/scripts/tests/test_trustbench.py`, focused contract tests

Local-only files, ignored by Git:

- `.elai_cc/benchmarks/harness_trust/upstreams/`, upstream checkouts
- `.elai_cc/benchmarks/harness_trust/venvs/`, one Python environment per suite
- `.elai_cc/benchmarks/harness_trust/results/`, reports, logs, workspaces, and normalized JSONL

ONE-TIME HEALTH CHECK

```bash
cd /home/user/teikoku/elai
python3.12 .elai_cc/scripts/trustbench.py list
python3.12 .elai_cc/scripts/trustbench.py doctor
```

`doctor` checks Python, Git, pinned checkout state, virtualenv state, imports, and Docker availability. It does not start a model or benchmark task.

REBUILD THE LOCAL SETUP

The current manifest pins the four cloneable repositories to exact commits. To create or restore the local checkouts and environments:

```bash
cd /home/user/teikoku/elai
python3.12 .elai_cc/scripts/trustbench.py bootstrap \
  --all \
  --python python3.12
```

Bootstrap installs the dependencies declared by each upstream repository into its own environment. It does not install native harness CLIs, Docker, provider credentials, or model weights.

Bootstrap refuses to reuse a checkout when its origin, commit, or working tree does not match the manifest. Do not delete or overwrite a mismatched checkout automatically. Inspect it first, then resolve the checkout deliberately.

DRY RUN ALL SUITES

Use this before any real harness execution:

```bash
cd /home/user/teikoku/elai
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/your/harness \
  --model fixed-model-id \
  --seed 7 \
  --all \
  --dry-run
```

The deterministic dry-run report is written to:

`.elai_cc/benchmarks/harness_trust/results/dry-run/report.json`

Expected statuses when no adapter commands are supplied:

- `adapter_required` for cloneable suites with no suite adapter
- `paper_only` for UnderSpecBench

A missing adapter is not a passing score.

CONNECT A HARNESS

Adapters are supplied at run time with:

```text
--adapter-command SUITE=COMMAND
```

The command is parsed without `shell=True`. These substitutions are available inside the command:

- `{harness}`, the value passed to `--harness`
- `{model}`, the value passed to `--model`
- `{seed}`, the value passed to `--seed`
- `{suite}`, the suite ID
- `{suite_root}`, the pinned upstream checkout
- `{workspace}`, the isolated workspace for this suite
- `{output}` or `{artifact}`, the JSONL artifact path the adapter must write

The runner also exports:

- `HARNESS_TRUST_SUITE`
- `HARNESS_TRUST_HARNESS`
- `HARNESS_TRUST_MODEL`
- `HARNESS_TRUST_SEED`
- `HARNESS_TRUST_WORKSPACE`
- `HARNESS_TRUST_OUTPUT`

A future adapter should invoke the supplied harness, run the selected upstream suite, use the upstream checker/scorer, and write at least one JSON object per line to `{output}`. A row may include a numeric `score`, but the umbrella report preserves it without aggregating it.

SAFE ADAPTER SHAPE

Replace `path/to/adapter.py` with the adapter you own:

```bash
cd /home/user/teikoku/elai
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/your/harness \
  --model fixed-model-id \
  --seed 7 \
  --suite openbench \
  --adapter-command 'openbench=python path/to/adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --execute
```

`--execute` is required for any adapter subprocess. Without it, use `--dry-run`.

RUN ONE SUITE

```bash
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/your/harness \
  --model fixed-model-id \
  --seed 7 \
  --suite harnessaudit \
  --adapter-command 'harnessaudit=python path/to/harnessaudit_adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --execute
```

Available suite IDs:

- `harnessaudit`
- `control-arena`
- `agent-safetybench`
- `openbench`
- `underspecbench`

`underspecbench` remains paper-only and cannot be executed by this runner.

RUN ALL SUITES WITH ADAPTERS

Supply one adapter command for every suite that is ready to run:

```bash
python3.12 .elai_cc/scripts/trustbench.py run \
  --harness /absolute/path/to/your/harness \
  --model fixed-model-id \
  --seed 7 \
  --all \
  --adapter-command 'harnessaudit=python path/to/harnessaudit_adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --adapter-command 'control-arena=python path/to/control_arena_adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --adapter-command 'agent-safetybench=python path/to/agent_safety_adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --adapter-command 'openbench=python path/to/openbench_adapter.py --harness {harness} --model {model} --seed {seed} --workspace {workspace} --output {output}' \
  --execute
```

A suite without a command is reported as `adapter_required`. The runner continues other suites under `--all`.

READ A REPORT

For a human-readable report:

```bash
python3.12 .elai_cc/scripts/trustbench.py report \
  .elai_cc/benchmarks/harness_trust/results/dry-run/report.json
```

For machine-readable JSON:

```bash
python3.12 .elai_cc/scripts/trustbench.py report \
  .elai_cc/benchmarks/harness_trust/results/dry-run/report.json \
  --json
```

Executed runs use a UTC timestamp directory below:

`.elai_cc/benchmarks/harness_trust/results/<run-id>/report.json`

Each suite has its own subdirectory containing its workspace, raw adapter JSONL, normalized JSONL, stdout log, and stderr log.

STATUS MEANINGS

- `cloneable`, the official source and exact commit are recorded
- `paper_only`, paper available but no verified official code checkout
- `adapter_required`, setup exists but no command was supplied
- `capability_missing`, checkout or virtualenv is not ready
- `dry_run_ready`, command constructed but not executed
- `completed`, adapter exited successfully and produced valid JSONL evidence
- `failed`, adapter exit, timeout, malformed JSONL, or empty artifact failure

SCORE RULE

The aggregate is intentionally disabled. These upstream suites measure different task populations, threat models, environments, and score semantics. A numeric field in one suite is not automatically comparable to a numeric field in another.

Do not publish a combined trust score until an adapter declares:

1. The exact score field and its external checker.
2. The fixed model, harness, seed, task set, and environment pins.
3. A justified weighting policy.
4. Evidence that all included suite scores have comparable meaning.

SAFETY RULES

- Start with `doctor` and `--dry-run`.
- Use isolated workspaces and never point the runner at a production repository or live infrastructure.
- Keep API keys and provider credentials outside the repository.
- Do not place secrets in adapter command arguments. Secret-looking command arguments are redacted in reports, but avoiding them is safer.
- Do not run `--execute` until the adapter’s checker, workspace, permissions, and model configuration are understood.
- ControlArena settings that use sandboxing require Docker at evaluation time.
- Do not execute UnderSpecBench from an unverified fork.
- The runner does not install or configure native harness CLIs for you.

LOCAL VERIFICATION

Run the focused contract tests after changing the runner or adapter boundary:

```bash
cd /home/user/teikoku/elai
python3.12 -m unittest discover \
  -s .elai_cc/scripts/tests \
  -p test_trustbench.py
python3.12 -m py_compile \
  .elai_cc/scripts/trustbench.py \
  .elai_cc/scripts/tests/test_trustbench.py
```

The full upstream benchmark suites are operational runs, not setup checks. Run them only after the harness adapter and credentials are ready.

TROUBLESHOOTING

`doctor` reports `missing_checkout`:

```bash
python3.12 .elai_cc/scripts/trustbench.py bootstrap --suite openbench --python python3.12
```

Replace `openbench` with the affected suite. Use the exact suite ID from `list`.

`doctor` reports `ref_mismatch`, `remote_mismatch`, or `dirty_checkout`:

Stop and inspect the checkout. Bootstrap intentionally refuses to overwrite it.

A run reports `adapter_required`:

Supply `--adapter-command SUITE=COMMAND` and use `--dry-run` first.

A run reports `capability_missing`:

Run `doctor`, then bootstrap only the affected suite.

A run reports `failed`:

Read the suite-specific `stdout.log` and `stderr.log`, validate that the adapter wrote non-empty JSONL to the `{output}` path, and check the upstream suite’s own requirements.

REFERENCE FILES

- Main README: `.elai_cc/benchmarks/harness_trust/README.md`
- Manifest: `.elai_cc/benchmarks/harness_trust/suites.json`
- Runner: `.elai_cc/scripts/trustbench.py`
- Contract tests: `.elai_cc/scripts/tests/test_trustbench.py`
