# P0: Opus FAST_FAIL_STORM — stale elai OAuth while max is fresh

## Symptom
- Opus head turns exit `rc=1` in ~5s; after 3 consecutive → `FAST_FAIL_STORM` → chain does **not** auto-restart.
- Smoke: `authentication_error / Invalid authentication credentials` (401).
- Overnight 2026-07-13 ~09:58Z: walk halted after turn 42.
- Manual: `/home/user/.claude/.credentials.json` `expiresAt` **past**, while `/mnt/c/Users/user/.claude/.credentials.json` (max) still ~8h remaining. Copying accessToken from max → WSL fixed smoke.

## Evidence
- `opus_supervisor._spawn_opus_turn` calls `token_refresh.refresh_if_needed(elai)` then `_sync_elai_token_from_max()` (lines ~499–512).
- `_sync_elai_token_from_max` (lines 450–480): only copies when `src.expiresAt > dst.expiresAt`; copies **accessToken + expiresAt only**, never refreshToken (by design: one refresh chain per client).
- `worker_providers.claude_env`: strips `ANTHROPIC_*` so base-cc reads plan OAuth from disk config dir (`~/.claude`).
- Headless base-cc does **not** auto-refresh; spawn-path sync is the safety net.
- Timer: `teikoku-token-refresh.service` only refreshes elai’s **own** refresh chain; if that chain is invalidated by interactive max refresh, elai reports remaining_secs but API rejects access tokens until max is copied.
- Observed secondary: missing/corrupt `~/.claude/.claude.json` also caused fast fail earlier same night (config-not-found); restored from backups.

## Root cause (ranked)
1. **Primary:** elai/WSL credential file lag behind max access token. Interactive sessions refresh **max**; that can invalidate or desync the elai-side access token. Spawn-path sync exists but:
   - Only runs if max `expiresAt` is **strictly greater** than local — if elai has a *future but already-invalid* expiresAt (provider kill), sync is skipped.
   - Or path not updated before long multi-hour turn burns past expiry mid-chain (sync is per-spawn; works, unless max itself wasn’t updated into the path base-cc reads).
   - Distro `~/.claude/.credentials.json` is the binding store; Windows elai copy may update while WSL home lags if only one dst fails.
2. **Secondary:** FAST_FAIL_STORM never retries after token repair (operator halt) — overnight guardian must force-sync + smoke + restart; smoke was wrong-or-fail-closed when 401 returned with `is_error:true` but non-empty JSON.
3. **Tertiary:** `.claude.json` corruption/missing causes ~same 5s fail; looks like auth in logs.

## Proposed fix
1. **`_sync_elai_token_from_max` hardened:**
   - Sync if max `expiresAt` is greater **OR** local is within `NEAR_EXPIRY` of now **OR** force flag `OPUS_TOKEN_FORCE_SYNC=1`.
   - Always ensure both dests: `token_refresh.ACCOUNTS["elai"]` **and** `Path.home()/".claude/.credentials.json"`.
   - After write, verify file mtime + that expiresAt > now + 60s; log event `OPUS_TOKEN_SYNC`.
2. **On FAST_FAIL path in `run_supervisor`:** before counting toward FAST_FAIL_LIMIT (or after first fast-fail), call force-sync + optional one soft retry of the spawn **once per chain** (`token_rescue_used` flag).
3. **`opus_walk_chain`:** on FAST_FAIL_STORM, do **one** force-sync + smoke_opus (cheap OK ping) + single restart attempt with backoff 60s before permanent halt. Cap: 1 rescue per 6h (file stamp).
4. **Guardian smoke_opus:** treat `is_error:true` + authentication_error as fail (already intended); ensure CLAUDE_CONFIG_DIR + config json existence check before smoke.

## Test plan
- Unit: mock max fresher / elai expired → sync writes both paths.
- Unit: elai expiresAt future but force → sync runs.
- Integration dry: delete WSL expires into past, call `_sync_elai_token_from_max`, assert expiresAt matches max.
- Regression: existing token_refresh tests still green.

## Risk
- Over-sync could thrash files; mitigate with force-once flag and expiresAt compare.
- Never copy refreshToken from max into elai (keep current ban).
