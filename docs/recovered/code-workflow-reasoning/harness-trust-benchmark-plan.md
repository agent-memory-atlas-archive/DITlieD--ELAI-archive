HARNESS TRUST BENCHMARK PLAN

SCOPE

Download and prepare the four verified cloneable upstream suites named by the user, record UnderSpecBench without inventing a repository, and add a local umbrella contract that lets a future harness run each suite with fixed model/task/environment parameters and produces one normalized report.

GROUNDING

- Existing project surface: `.elai_cc/crates/cli/src/main.rs` already exposes many `elai bench` commands, and `.elai_cc/benchmarks/registry.json` is the versioned benchmark contract.
- HarnessAudit source: https://github.com/UCSB-AI/HarnessAudit, current `main` ref verified with `git ls-remote`: `6317162590aeeb1c8dde32b880ac199933343e4a`, MIT, Python >=3.11, editable install with `.[dev,oai]` from its README and `pyproject.toml`.
- ControlArena source: https://github.com/UKGovernmentBEIS/control-arena, current `main` ref verified with `git ls-remote`: `3a378ed00eb1f6b8e893e39215d749688126c9cb`, MIT, Python >=3.11, source install declared by its `pyproject.toml`; Docker is required for many sandbox settings according to its official documentation.
- Agent-SafetyBench source: https://github.com/thu-coai/Agent-SafetyBench, current `main` ref verified with `git ls-remote`: `74feea8de601b3a1449a93fcf70017fe61556f73`, MIT, requirements file declares `transformers`, `torch`, and `openai`.
- OpenBench source: https://github.com/minghinmatthewlam/openbench, current `main` ref verified with `git ls-remote`: `9364ebaf6279bb533cac073fd2e67f4b7d67d8d9`, MIT, Python >=3.11, stdlib-only runtime, editable install exposes `obench`.
- UnderSpecBench paper: https://arxiv.org/html/2607.02294. The paper is public and describes the benchmark, but the official-source search did not produce a verified cloneable repository URL. The setup must report this as `paper_only`, not clone an unverified project.

DESIGN DECISION

Use a thin stdlib Python umbrella runner and a JSON manifest under `.elai_cc/benchmarks/harness_trust/`, not a new Rust `elai bench` family. The existing Rust bench surface is in-process and fixture-oriented; the four upstream projects are separate Python applications with different entrypoints, environments, and result formats. A Rust wrapper would add cross-language process management without eliminating those differences. The umbrella runner will own acquisition, environment health, adapter dispatch, run metadata, and normalized reporting; upstream scoring remains upstream-owned.

ALTERNATIVES CONSIDERED

1. Rust `elai bench trustbench` subcommand. Benefit: one native CLI. Cost: changes the already large CLI dispatcher and still needs Python environment/process/result handling. Reject for this setup slice.
2. Separate manual checkout instructions only. Benefit: minimal code. Cost: no pinned matrix, no reproducible bootstrap, no common dry-run contract, and no honest availability report. Reject because it does not satisfy “plug in a harness later.”

PHASE 1: ACQUIRE AND ISOLATE UPSTREAMS

Deliverables:
1. `.elai_cc/benchmarks/harness_trust/suites.json`: suite IDs, official URLs, pinned refs, licenses, status, environment paths, setup metadata, and run adapter metadata.
2. `.elai_cc/benchmarks/harness_trust/.gitignore`: ignores local upstream checkouts, virtual environments, and runtime results while keeping the manifest and fixtures tracked.
3. `.elai_cc/scripts/trustbench.py`: `list`, `bootstrap`, and `doctor` commands, using only Python standard library for orchestration.

PRE-GATES:
- `python3 --version` reports Python 3.11 or newer.
- `git --version` succeeds.
- The four recorded refs still resolve from their official remotes.
- The project worktree is clean before acquisition.

ADVERSARIAL CHECKS:
- Existing checkout at the expected path has the wrong remote or commit: bootstrap must fail closed instead of reusing it.
- A suite directory is present without a valid Git checkout: doctor must report the suite unavailable, not claim readiness.
- A missing optional Docker or harness CLI must be reported as an unmet capability, not installed or silently ignored.

PHASE 1 DONE WHEN:
`python3 .elai_cc/scripts/trustbench.py list` shows four pinned cloneable suites plus UnderSpecBench as `paper_only`, and `doctor` identifies every missing optional capability without network or model execution.

PHASE 2: BOOTSTRAP AND ADAPTER CONTRACT

Deliverables:
1. Extend `trustbench.py` with isolated per-suite virtualenv creation and repository-declared dependency installation.
2. Add a normalized JSONL envelope contract for run metadata, suite status, harness identity, model identity, seed, workspace, raw artifact path, and score fields.
3. Add deterministic unit tests under `.elai_cc/scripts/tests/` covering manifest validation, command construction, path containment, missing-adapter handling, and JSONL normalization.

SETUP RULES:
- HarnessAudit uses its declared editable `.[dev,oai]` install and installs vendored ClawTeam only when that directory exists.
- ControlArena uses its source-declared editable install; Docker is checked but not installed by the bootstrapper.
- Agent-SafetyBench uses its repository-declared requirements file.
- OpenBench uses its source-declared editable install.
- No API key values are written; `.env` and provider credentials remain user-managed.
- No native harness CLI is installed automatically; the user supplies the harness executable or adapter later.

ADAPTER CONTRACT:
A suite adapter must accept a fixed model ID, harness command/path, seed, isolated workspace, and output directory, then emit either a normalized JSONL row or an explicit `adapter_required`, `capability_missing`, `paper_only`, or `failed` status. The runner must never convert an absent score into a passing score.

ADVERSARIAL CHECKS:
- A command tries to write outside the declared workspace/output root: reject before execution.
- A subprocess exits nonzero or emits malformed JSON: preserve the failure status and stderr path.
- A suite has no verified harness adapter: return `adapter_required` and continue other suites under `--all`.

PHASE 2 DONE WHEN:
`python3 .elai_cc/scripts/trustbench.py bootstrap --suite <id>` creates only the target suite environment, and a deterministic adapter-contract test suite passes without API keys, Docker, or model calls.

PHASE 3: UMBRELLA RUN AND REPORT

Deliverables:
1. Add `run` and `report` commands to `trustbench.py` with `--harness`, `--model`, `--seed`, `--suite`, `--all`, `--dry-run`, and output-directory controls.
2. Add `.elai_cc/benchmarks/harness_trust/README.md` documenting bootstrap, doctor, dry-run, adapter registration, safety boundaries, and the UnderSpecBench paper-only caveat.
3. Add one tracked integration entry to the existing benchmark registry only if its schema can represent an external, non-Rust suite without changing existing rows; otherwise keep the umbrella registry self-contained and document why.

REPORT CONTRACT:
- One JSON report contains run metadata, fixed-variable pins, per-suite statuses, raw artifact references, normalized scores when available, and an aggregate only over suites with valid comparable scores.
- `--dry-run` never starts a model, native harness, Docker container, or external task.
- `--all` includes unavailable and paper-only suites in the report so coverage is visible.
- Aggregate scoring is disabled unless comparable numeric scores and the declared weighting policy are present; no invented trust score.

ADVERSARIAL CHECKS:
- Same model/task pins differ across suites: report a comparability warning and do not aggregate.
- A harness self-reports success while the external checker fails: use the checker/status, not the self-report.
- A run directory contains secrets or `.env` files: reject or scrub the report reference and never print contents.

PHASE 3 DONE WHEN:
A dry-run with a placeholder harness path produces a deterministic JSON report that lists all five requested suites, refuses to fabricate scores, and prints the exact command shape for plugging in a real harness.

UNIT TESTS

- Manifest schema and pinned-ref validation, no network.
- Path containment and secret-name redaction, no subprocess execution.
- Deterministic dry-run report shape, no model/API/Docker.
- Failure propagation for nonzero subprocess and malformed result JSON.

OPERATIONAL CHECKS

- Clone each verified upstream at its recorded ref.
- Build each isolated environment.
- Run only repository import/CLI help or documented no-model doctor checks.
- Run actual benchmark tasks only after the user supplies a harness, model, credentials, and any required Docker/runtime approval.

TOP RISKS

1. Dependency weight: ControlArena and Agent-SafetyBench declare large ML/sandbox stacks. Mitigation: isolated environments, per-suite bootstrap, no automatic Docker installation, and no cross-suite dependency sharing.
2. Harness comparability: the suites do not expose a common score or identical environment. Mitigation: preserve per-suite raw results and disable aggregate scoring until a declared adapter supplies comparable evidence.
3. UnderSpecBench availability: no official repository was verified. Mitigation: record the paper and paper-only status, never clone or execute an unverified fork.
