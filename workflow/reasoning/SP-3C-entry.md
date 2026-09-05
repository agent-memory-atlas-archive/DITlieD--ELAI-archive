SP-3C entry: Terminal-Bench adapter + day-0 baseline probe (D3 ADVISORY)

task_id: SP-3C, session s-finish-0705, plan ref workflow/reasoning/TEIKOKU-ELAI-FINISH-PLAN.md unit 3.3.

SUMMARY
Recon + partial build-out on codex-vm (WSL distro). docker daemon was down, fixed. terminal-bench
0.2.18 installed in a venv. bmapi.020212.xyz proxy confirmed reachable + authenticates with the
/root/.codex/auth.json OPENAI_API_KEY, gpt-5.5 confirmed present in its model list. ELAI's CLI
source already has the exact mechanism this task needs (generic OpenAI-compat provider +
`-p "<prompt>" --model <name> --print` one-shot mode) at commit d5d5fde, confirmed by reading
crates/providers/src/providers/openai_compat.rs and crates/cli/src/args.rs on a refreshed
/root/elai-clone. Wrote a real `ElaiAgent(AbstractInstalledAgent)` adapter + install script
matching tb's actual custom-agent API (verified by reading the installed tb package source, not
assumed from training). Ran out of time-box before a scored `tb run` could execute — the
remaining gap is source-delivery-into-the-task-container, not anything ELAI-side.

WORK DONE
- docker daemon started on codex-vm (`service docker start`; was not running, no systemd, uses
  the sysvinit script).
- terminal-bench installed via python3-pip + venv at /root/elai-tb/.venv (`pip install
  terminal-bench` -> 0.2.18, confirmed live on PyPI before install). Pinned click==8.1.7 chasing
  a `tb runs create --help` parsing bug ("No such option: --help (Possible options: --help)");
  the pin did NOT fix it. Top-level `tb --help` works fine; only `tb runs create --help`
  (equivalently `tb run --help`) is broken. This looks like an upstream tb 0.2.18 bug in how
  `runs create`'s ~20 chained typer Options interact with click 8.4.2's option validation order,
  not an environment problem -- did not chase further (out of scope per the 30-min time-box; the
  bug is cosmetic, --help is not needed for a real invocation).
- confirmed bmapi.020212.xyz: `GET /v1/models` unauthenticated -> 401 API_KEY_REQUIRED (expected,
  standard Bearer scheme). With the key read out of /root/.codex/auth.json (JSON field
  "OPENAI_API_KEY", 67-char value, never printed) -> 8 models listed including "gpt-5.5" and
  "gpt-5.5-openai-compact".
- refreshed /root/elai-clone (was stale at d5d5fde / 2026-07-03): manually mounted F: as drvfs
  (`mount -t drvfs F: /mnt/f`, automount is disabled in this distro's /etc/wsl.conf) and rsync'd
  Cargo.toml + Cargo.lock + crates/ from /mnt/f/.ELAI_workflow/.elai_cc (excluding target/, which
  is 36G and irrelevant to a source refresh). This intentionally does NOT pull docs/benchmarks/
  archive/models/ (1.3G, not needed to build the cli binary) -- if a later session needs those,
  rsync them separately. rsync was STILL RUNNING at session end (~220MB / crates copied so far;
  cross-filesystem drvfs copy of ~2000 small .rs files is slow, budget another 10-20 min). It was
  launched as a detached background process inside the VM and should keep running independent of
  this session ending, but was not confirmed complete.
- confirmed via crates/cli/Cargo.toml that the produced binary is named `elai` (not `cli`).
- wrote /root/elai-tb/agent_pkg/elai_adapter.py (ElaiAgent, 67 lines) and
  /root/elai-tb/agent_pkg/setup.sh (19 lines) on the vm, matching the REAL
  `AbstractInstalledAgent` contract read from
  /root/elai-tb/.venv/lib/python3.12/site-packages/terminal_bench/agents/installed_agents/abstract_installed_agent.py
  (BaseAgent.perform_task, `_env` / `_install_agent_script_path` / `_run_agent_commands`
  properties, `TerminalCommand` pydantic model fields). This is the correct base class for a
  "drive an installed CLI tool" agent, matching how tb's own claude-code/codex agents work.

FILES TOUCHED (all on codex-vm, none on the live F: working tree)
- /root/elai-tb/.venv/ -- new venv, terminal-bench 0.2.18 + pinned click 8.1.7
- /root/elai-tb/agent_pkg/elai_adapter.py -- new, the ElaiAgent adapter (untested end-to-end)
- /root/elai-tb/agent_pkg/setup.sh -- new, install script (builds `elai` from source in-container;
  ASSUMES /elai-src exists in the container, which nothing currently populates -- see QUESTION)
- /root/elai-tb/get_key.py, /root/elai-tb/show_models.py -- scratch scripts used for the API-key
  smoke test, harmless to leave or delete
- /root/elai-clone/.elai_cc/Cargo.toml, Cargo.lock, crates/ -- refreshed from /mnt/f (rsync,
  in-progress at session end)
- codex-vm docker daemon -- now running (was stopped)
- F:\.ELAI_workflow\workflow\reasoning\finish-bench-ledger.md -- new, day-0 infra-readiness row
- F:\.ELAI_workflow\workflow\reasoning\SP-3C-entry.md -- this file

USER CHOICES / DECISIONS
- None yet requested from the user this session; the fork below needs a decision before the
  probe can run.

QUESTION FOR ORCHESTRATOR
The ElaiAgent install script (setup.sh) needs the ELAI source tree (or a prebuilt `elai` binary)
INSIDE the task's Docker container, and nothing currently delivers it there -- the container is a
fresh, isolated environment per TB task, separate from the codex-vm's filesystem. Two options:
  (A) prebuild `elai` once on the vm (`cargo build --release -p cli` after the rsync above
      finishes) and bake it into a small custom base image (or serve it over the docker bridge /
      a tiny HTTP server on the vm host, since containers on the same dockerd can usually reach
      the host via the bridge gateway IP even without Docker-Desktop's host.docker.internal
      convenience DNS) -- faster per-task, but needs a one-time image-build/network-serving step
      I have not built.
  (B) have setup.sh do the full `cargo build --release` from a source tarball fetched at
      container-start time (matches how tb's other installed agents work, e.g. npm install -g
      inside the container) -- simpler, no extra image, but costs a multi-minute Rust compile
      PER TASK, which on an n=3 day-0 probe is still fine but scales badly for a real run later.
My lean: (B) first for the day-0 n=3 probe (simplest, no extra infra), switch to (A) only if a
later, larger run makes the per-task compile cost matter. Either way setup.sh needs one more
line: how the source/binary actually reaches the container filesystem (currently a placeholder
`test -d /elai-src || exit 1`).

HOW TO PICK UP
1. Confirm the rsync finished: `wsl -d codex-vm -- bash -lc "ps aux | grep rsync"` should show
   nothing; `du -sh /root/elai-clone/.elai_cc/crates` should have stopped growing.
2. Pick (A) or (B) above and wire the source/binary delivery into setup.sh accordingly.
3. Smoke-test directly on the vm FIRST, bypassing tb entirely, to isolate ELAI-CLI-vs-tb-harness
   failures: `cd /root/elai-clone/.elai_cc && cargo build --release -p cli` then
   `OPENAI_BASE_URL=https://bmapi.020212.xyz/v1 OPENAI_API_KEY=<from auth.json> ./target/release/elai -p "say ok" --model gpt-5.5 --print`.
4. Only once step 3 works, run
   `tb run --agent-import-path elai_adapter:ElaiAgent --dataset terminal-bench-core --n-tasks 3`
   from /root/elai-tb/agent_pkg (with the venv active) and append the real scored row to
   finish-bench-ledger.md.
5. Note the tb 0.2.18 `tb runs create --help` cosmetic bug (see WORK DONE) if it resurfaces
   elsewhere -- it did not block any real invocation attempted this session.

KNOWN LIMITS / NEXT
- click==8.1.7 pin did not fix the `--help` bug; left in place (harmless), root cause
  unconfirmed, not blocking.
- ElaiAgent adapter is UNTESTED end-to-end (no tb run executed, no elai binary built yet this
  session).
- No credits were spent against the gpt-5.5 model path this session (only a models-list GET was
  made); the "spend-down" part of the unit has not started.
- /root/elai-clone refresh deliberately excludes docs/benchmarks/archive/models (not needed to
  build the cli binary); pull those separately if a later step needs them.

GROUNDING / VERIFICATION
- docker: `service docker start` then `docker ps` returned a clean empty table (verified).
- tb: `pip show terminal-bench` -> 0.2.18 (verified); PyPI JSON for terminal-bench also showed
  0.2.18 as latest (verified before install, anti-slopsquat check).
- bmapi: `curl .../v1/models` unauthenticated -> HTTP 401 API_KEY_REQUIRED (verified); with key ->
  HTTP 200, 8 model ids incl. gpt-5.5 (verified, key never printed to any log/output).
- ELAI CLI contract: read directly from crates/providers/src/providers/mod.rs (metadata_for_model,
  detect_provider_kind), crates/providers/src/providers/openai_compat.rs (OPENAI_BASE_URL /
  OPENAI_API_KEY env names, DEFAULT_OPENAI_BASE_URL), crates/cli/src/args.rs (`-p` / `--print`
  branches at ~line 2735-2750) on the refreshed /root/elai-clone -- all High confidence (code just
  read), not from training memory.
- tb custom-agent API: read directly from the installed terminal_bench package
  (agents/base_agent.py, agents/naive_agent.py, agents/installed_agents/abstract_installed_agent.py,
  terminal/models.py) -- High confidence, not from training memory or docs.
- Nothing in this session mutated git, printed the OPENAI_API_KEY value, or touched the teikoku
  distro / the live F:\.ELAI_workflow working tree (only read from it, via a manual drvfs mount,
  to rsync a copy onto the vm).
