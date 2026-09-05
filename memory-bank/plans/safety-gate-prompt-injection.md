# Plan: Safety gate between Codex and ELAI code

## Goal
Add a defensive guard layer that detects prompt-injection attempts and suspicious content flowing between the Conductor and the Codex worker, reducing the blast radius of a compromised or misbehaving proxy/model.

## Scope
- Outgoing prompt validation: scan the task payload + context packet before it is handed to Codex.
- Incoming output validation: scan Codex stdout/stderr after the run for signs of injection or malicious instructions.
- Fail closed: trip → task becomes `BLOCKED` with a clear reason, no worktree mutation is accepted.
- Does not replace existing redaction, allowed-write-set, or verifier gates; it complements them.

## Deliverables
1. `conductor/guard.py` — `PromptGuard` and `OutputGuard` with pattern sets and severity scoring.
2. `conductor/worker.py` — integrate guards into `WorkerRunner.run_task` before dispatch and after return.
3. `tests/test_guard.py` — unit tests covering prompt injection, dangerous instruction, and benign content.
4. Run unit tests and update progress.

## Tests
- Unit: guard passes on normal task payload.
- Unit: guard blocks payload containing "ignore previous instructions".
- Unit: guard blocks output containing "rm -rf /".
- Regression: full worker pipeline still completes on clean payload.

## Checkpoint
Phase done when `python3 -m unittest discover -s tests` passes and a synthetic injected payload is rejected.
