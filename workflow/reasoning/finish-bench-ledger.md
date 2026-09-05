Finish-bench ledger (Terminal-Bench spend-down of bmapi gpt-5.5 credits, D3 ADVISORY lane, unit 3.3 of TEIKOKU-ELAI-FINISH-PLAN.md)

Format: one row per probe/run. Never a done-gate; a benchmark can never wedge the queue (D3).

Reference row: codex-cli + gpt-5.5 = 83.4% Terminal-Bench 2.1 (morphllm.com, cited by plan; not independently re-verified here).

---

2026-07-06 session s-finish-0705 task SP-3C -- DAY-0 INFRA RECON (no scored tasks run)
adapter version: v0 (elai_adapter.py, AbstractInstalledAgent subclass; UNTESTED end-to-end)
dataset: n/a (probe not reached)
n: 0
pass/fail: n/a
credits before/after: not measured (no proxy call billed against the gpt-5.5 model path this session; only a GET /v1/models auth check was made against bmapi.020212.xyz, which listed 8 models including gpt-5.5 and gpt-5.5-openai-compact)
caveat: this is an infra-readiness row, not a benchmark result. See workflow/reasoning/SP-3C-entry.md for the full recon trail. Verified this session: codex-vm docker daemon (was down, started via `service docker start`), terminal-bench 0.2.18 installed in /root/elai-tb/.venv (pip, click pinned 8.1.7 -- tb's own `--help` flag is broken on `tb runs create --help` regardless of click pin, cosmetic only, does not block real invocation), bmapi.020212.xyz reachable and the OPENAI_API_KEY from /root/.codex/auth.json authenticates (401 without key, model list returned with key), ELAI's generic OpenAI-compat provider path (crates/providers/src/providers/openai_compat.rs, env OPENAI_BASE_URL/OPENAI_API_KEY, ProviderKind::OpenAi fallback in detect_provider_kind when no anthropic/xai key present) and the CLI's one-shot `-p "<prompt>" --model <name> --print` non-interactive mode (crates/cli/src/args.rs ~line 2735-2750) are both present in the source. NOT verified: an actual `elai -p ... --model gpt-5.5` round trip against bmapi (no live call made this session); the ElaiAgent adapter has never been driven through a real `tb run`.
next: (1) finish the /root/elai-clone rsync refresh (crates/ was ~220MB copied and still running via drvfs at session end -- cross-filesystem copy from /mnt/f is slow, budget 10-20 more min); (2) `cargo build --release -p cli` on the vm to get a working `elai` binary and smoke-test `OPENAI_BASE_URL=https://bmapi.020212.xyz/v1 OPENAI_API_KEY=<key> elai -p "say ok" --model gpt-5.5 --print` directly on the vm (bypassing tb) before trusting the adapter; (3) resolve source-delivery-into-container for the ElaiAgent's install script (currently assumes /elai-src exists inside the task container, which is NOT wired -- either bake a prebuilt `elai` binary into a base image tb can pull, or serve the source tree over the docker bridge from the vm host); (4) then run the real `tb run --agent-import-path elai_adapter:ElaiAgent --dataset terminal-bench-core --n-tasks 3` probe and append a real scored row here.
