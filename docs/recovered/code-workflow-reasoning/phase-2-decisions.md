PHASE 2 DECISIONS

WHAT

Make each verified upstream runnable in an isolated Python 3.12 environment and define the external adapter/result boundary without executing benchmark tasks.

KEY DECISIONS

- Install only the upstream-declared package metadata: HarnessAudit editable `.[dev,oai]` plus its vendored ClawTeam package, ControlArena editable source, Agent-SafetyBench `requirements.txt`, and OpenBench editable source.
- Keep environments separate because the upstream dependency graphs differ materially and Agent-SafetyBench includes the ML runtime stack.
- Treat adapter commands as explicit caller input, parse them without a shell, prepend the selected suite virtualenv to PATH, and require `--execute` for subprocess execution.
- Normalize only valid JSON object lines and preserve suite-native raw rows; malformed or empty artifacts are failures.

EXPECTED TEST OUTCOME

- Four checkouts match their declared remotes and refs.
- Four virtualenvs report Python 3.12 and import checks pass.
- Focused contract tests pass with no network, model, Docker, or upstream task execution.

UNKNOWNS

- Optional native harness binaries and API credentials are intentionally absent until supplied by the user.
- ControlArena settings may need Docker at actual run time even though the Python package imports successfully.
- Agent-SafetyBench dependency installation may select a large platform-specific torch/CUDA distribution from its unpinned requirements.
