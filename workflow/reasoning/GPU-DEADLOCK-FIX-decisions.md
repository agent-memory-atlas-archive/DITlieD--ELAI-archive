GPU-DEADLOCK-FIX — Decision Doc (retroactive)
=============================================
Phase: GPU-DEADLOCK-FIX
Session: s-27276

CONTEXT
-------
Investigation report at workflow/sessions/s-27276/gpu-deadlock-investigation.md
identified 3 root causes ranked by evidence:
  1. WSL2 process boundary (Windows kill cannot reach Linux python in VM)
  2. No GPU cleanup on exit (lifespan teardown skips del model / synchronize / empty_cache)
  3. bitsandbytes NF4 double-quant interruptible mid-kernel

WHAT
----
Permanent fix for recurring GPU deadlock that previously required host reboots:
- _graceful_gpu_teardown() helper in server.py (idempotent, logs each step,
  catches partial-init).
- Lifespan teardown calls pool drain FIRST then teardown (use-after-free fix).
- NO standalone signal handler — uvicorn ≥ 0.29 overwrites them; lifespan teardown
  runs natively under uvicorn's SIGTERM handling.
- start-sidecar.bat: pre-launch `pkill -9` reaps stale orphans from prior crashes.
- stop-sidecar.bat: NEW; SIGTERM (5s) then SIGKILL inside WSL2 to clean up.

KEY DECISIONS
-------------
D1. Teardown order: pools drain FIRST (wait=True), THEN _graceful_gpu_teardown.
    Rationale: a thread mid-`model.generate()` would otherwise see its model
    deleted out from under the running CUDA kernel.
D2. Skip standalone signal handler in `__main__`.
    Rationale: uvicorn ≥ 0.29 installs its own SIGTERM/SIGINT handlers; ours
    would be dead code. The lifespan path is the supported mechanism and runs
    under uvicorn's signal handling.
D3. Reaper in start-sidecar.bat (`pkill -9 -f elai_inference.server`) before launch.
    Rationale: handles the case where a prior run was killed via SIGKILL or a
    WSL VM hang and left a process holding VRAM.
D4. WSL-side kill (`wsl -d Ubuntu -- pkill`) in stop-sidecar.bat.
    Rationale: `taskkill /F` on Windows cannot reach the Linux process; it
    targets the wsl.exe stub.
D5. Drop `_engine.model = None` line.
    Rationale: violates non-Optional type; redundant since `_teardown_done`
    already guards re-entry and `_engine = None` nulls the whole ref.

EXPECTED TEST OUTCOMES
----------------------
- python ast.parse on server.py: PASS.
- Future sidecar runs: graceful Ctrl+C / kill -TERM releases VRAM cleanly.
- pkill from inside WSL2: works.
- taskkill from Windows: not used (intentional).

UNKNOWNS / KNOWN LIMITATIONS
----------------------------
- SIGKILL bypasses lifespan teardown. Recovery: `wsl --terminate Ubuntu`. No
  in-process fix possible.
- 30-second concurrent.futures deadline (investigation D.3) NOT implemented;
  unbounded `wait=True` could block shutdown indefinitely if a generate() call
  is stuck. Documented as follow-up.
- bitsandbytes NF4 mid-kernel interrupt: not addressed; out of scope.

ROLLBACK
--------
git restore on the 3 modified files.

ACCEPTANCE
----------
A1: server.py ast.parse PASS.
A2: lifespan teardown order: pools first, then GPU.
A3: no signal.signal calls inside __main__ that compete with uvicorn.
A4: stop-sidecar.bat exists and is idempotent.
A5: start-sidecar.bat reaps before launch.
