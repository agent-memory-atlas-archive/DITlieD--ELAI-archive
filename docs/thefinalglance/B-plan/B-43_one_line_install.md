# B-43: One-Line Install (curl-pipe-bash / iwr-pipe-iex, Win64 + Linux x86_64)
Plan ID: B-43
Title: one_line_install
Status: PLANNED
Tier: HEAVY
GRC budget: 8/1500 per topic
Estimated complexity: 7 phases, 3-5 sessions, ~12-18 new files, ~3-5 extended files
Predecessor plans: B-37 (RuntimeBackend dispatch — installer routes hardware to the right backend),
  B-42 (cosmos/ tier — installer mkdir's ~/.elai/cosmos/ on first run), C-28 (install_dev_tools.sh
  prior-art for ToolchainMissing pattern + cargo-mutants/mutmut idempotent install).
Successor plans (DEFERRED from B-43 by user-locked decisions, do not re-litigate in scope):
  B-44 signed_releases (Apple notarization + Windows EV cert beyond GH attestations),
  B-45 dashboard_installer (Tauri v2 .msi/.dmg/AppImage bundles + per-OS desktop integration),
  B-46 package_manager_distribution (Homebrew tap, scoop manifest, AUR PKGBUILD, winget),
  separate plan auto-updater (delta updates + signature pinning),
  E-51 MLX adapter (Apple Silicon sidecar — unblocks aarch64 / macOS arm64 install path),
  separate plan Linux desktop integration (.desktop / .deb / .AppImage targets).

## NOTES (LOCKED architectural decisions, do not re-debate)

- LOCK-1: name collision — `BootstrapPlan` in crates/core/src/bootstrap.rs is the CLI-startup
  bootstrap (CliEntry / FastPathVersion / etc) per E48_FOUR_PRE_EXISTING_NAME_COLLISIONS
  taxonomy. The new host-installer artifact in crates/cli/src/install.rs is named
  `HostInstaller` (NOT BootstrapPlan, NOT InstallPlan). Wrap-not-rename — leave bootstrap.rs
  alone, only add the new struct.
- LOCK-2: sidecar boot command set is LOCKED from REMOTE-SIDECAR.md:69-76. Installer reuses
  the exact form (`ELAI_FORCE_SDPA=1 SEARXNG_URL=http://127.0.0.1:8889 python server.py
  --model huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated --port 8100 --no-dola
  --turboquant-bits 0 --no-steering --load-in-4bit`). No fork, no flag drift.
- LOCK-3: POSIX installer mirrors install-sidecar.sh's 8-step idempotent pattern (env-probe,
  venv, pip baseline, torch fallback, deps, CUDA smoke, HF download with cache-skip,
  footprint). Re-runnable. Partial-resume state at `~/.elai/install-state.json`.
- LOCK-4: Tauri dashboard install is DEFERRED to B-45_dashboard_installer. B-43 ships CLI
  + sidecar runtime + doctor only. The dashboard binary is NOT downloaded or staged in B-43.
- LOCK-5: sidecar is NOT auto-booted post-install (HARDWARE-HEAVY rule). Install ends with
  `elai doctor` exit 0 plus a printed next-step hint: ``run `elai sidecar boot --local`
  to start the local model, or set ANTHROPIC_BASE_URL to remote tunnel``.
- LOCK-6: anti-slopsquatting — every URL in this plan has a PRE_GATE verification row.
  Every HF repo ID has a `WebFetch huggingface.co/<repo>` command spelled out before the
  Phase that consumes it. No "from training memory" URLs.
- LOCK-7: one commit at end of plan in maintainer voice per .claude/rules/commit-cadence.md
  (lowercase i, no em-dashes, no Co-Authored-By trailer, no robotic prefix). No per-phase
  commits. The commit lands ONCE after Phase 7 ships and EXIT_CRITERIA is satisfied.

## RESOLVED open questions (user-locked, baked into plan, do not re-prompt)

- Q1 RESOLVED: hand-rolled GH Actions release matrix. NO cargo-dist. NO new Rust deps.
  Matrix is bounded to <=30 lines per OS entry, sample cribbed from canonical rust GH
  Actions starter; cargo-dist adoption left open as a future follow-up (B-46) only if
  maintenance burden surfaces in production.
- Q2 RESOLVED: HARD ERROR for Apple Silicon (macOS arm64) and Linux aarch64. install.sh /
  install.ps1 detect arch, print the structured error `ELAI local sidecar unsupported on
  aarch64 / Apple Silicon until E-51 MLX adapter ships. Use remote sidecar tunnel (see
  workflow/remote-setup/REMOTE-SIDECAR.md) or wait for E-51.` and exit 1. DO NOT fall back
  to cloud-only-mode silently — that hides a real platform gap. Hard error per user.
- Q3 RESOLVED: native Python venv default for SearXNG (port 8889 per REMOTE-SIDECAR.md
  canonical). Docker is NOT supported in B-43 MVP. The `--searxng-docker` flag is DROPPED
  entirely — single canonical path, no dual-topology drift. scripts/start-searxng.sh's
  Docker pattern is orthogonal and stays out of B-43 install flow.
- Q4 RESOLVED: blake3 CHECKSUMS.blake3 manifest + GH attestations (sigstore-style). Apple
  notarization + Windows EV cert deferred to B-44_signed_releases.
- Q5 RESOLVED: pure shell scripts. install.sh (POSIX, `bash -euo pipefail`) +
  install.ps1 (PowerShell 7+, `$ErrorActionPreference="Stop"`). NO second-hop bootstrap
  binary. Matches rustup / ollama / uv / bun canonical pattern.
- Q6 RESOLVED: install-time eager model staging by default. Flag `--skip-models` for
  CI / corp-firewalled / Docker-image-baking workflows. First /task is fast (models
  already on disk).

## PRE_GATE — plan-level URL / dependency verification

Every URL fetched or HF repo touched by the installer is verified by the executor BEFORE
the consuming Phase begins. Anti-slopsquatting HARD RULE per CLAUDE.md.

```bash
# 1. GitHub release host (PLACEHOLDER-RESOLVED-AT-RELEASE — needs real org slug from user)
#    Tracked as pending-confirm in B-43-decisions.md
echo "PLACEHOLDER-RESOLVED-AT-RELEASE: https://github.com/<org>/elai"

# 2. HF repos (sidecar models — REMOTE-SIDECAR.md canonical)
curl -sfI https://huggingface.co/huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated >/dev/null
curl -sfI https://huggingface.co/huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated >/dev/null

# 3. HF repos (ONNX staging — stage_onnx_models.py recipes)
curl -sfI https://huggingface.co/microsoft/graphcodebert-base >/dev/null
curl -sfI https://huggingface.co/ibm-granite/granite-embedding-30m-english >/dev/null
curl -sfI https://huggingface.co/cross-encoder/ms-marco-MiniLM-L12-v2 >/dev/null

# 4. SearXNG upstream (Phase 5 clones / pip-installs)
git ls-remote https://github.com/searxng/searxng HEAD >/dev/null

# 5. Python 3.12 install reference (Phase 5 ToolchainMissing structured-error target)
curl -sfI https://python.org/downloads/release/python-31210/ >/dev/null
```

Each Phase that consumes one of these MUST cite the row above in its own Reads block.
PLACEHOLDER-RESOLVED-AT-RELEASE rows do NOT block PLANNED status but MUST be resolved
before SHIPPED status (see EXIT_CRITERIA).

## WIRING_CONTRACT (plan-level)

produces:
  - scripts/install/install.sh (POSIX entrypoint)
  - scripts/install/install.ps1 (PowerShell 7+ entrypoint)
  - scripts/install/elai_install_lib.sh (POSIX shared lib: detect_*, fetch_*, verify_blake3, render_config)
  - scripts/install/elai_install_lib.ps1 (PowerShell shared lib mirroring elai_install_lib.sh)
  - scripts/install/CHECKSUMS.blake3 (manifest, populated by Phase 1 release.yml)
  - scripts/install/elai_cc.toml.template (rendered into project root by Phase 6)
  - crates/cli/src/install.rs (NEW — `HostInstaller` struct + re-entry from `elai install`)
  - crates/cli/src/doctor.rs (EXTENDED — adds 4 new DiagnosticCheck functions)
  - crates/cli/src/args.rs (EXTENDED — adds `CliAction::Install` variant + parse_install_subcommand)
  - crates/cli/src/commands.rs (EXTENDED — adds "install" to Discovery & debugging group)
  - .github/workflows/release.yml (NEW — build matrix + blake3 manifest + GH attestations)
  - .elai_cc/scripts/stage_onnx_models.py (EXTENDED — adds blake3 verify + CHECKSUMS.blake3 author)
  - ~/.elai/install-state.json (runtime artifact — partial-resume state machine)

consumes:
  - elai_providers::vram_tier::detect_vram_tier (vram_tier.rs:7-92 — installer reads + renders into toml)
  - elai_providers::vram_tier::VramTier enum (Vram8Gb / Vram12Gb / Vram16Gb / Vram24Gb / Vram48Gb / Cloud)
  - elai_cli::doctor::DiagnosticCheck shape + DiagnosticLevel (Ok/Warn/Fail) + DoctorReport::has_failures
  - elai_cli::doctor::run_doctor (existing entrypoint — new checks compose in)
  - server.py FA2 detection env (ELAI_FORCE_SDPA, SEARXNG_URL) — installer never edits server.py
  - B-37 RuntimeBackend dispatch — installer writes default backend per detected hardware
  - B-42 ~/.elai/cosmos/ path — installer mkdir's the directory; population is B-42 worker's job

reachable:
  - from `curl -fsSL https://elai.dev/install.sh | bash` (POSIX one-line entry; HTTPS-only)
  - from `iwr https://elai.dev/install.ps1 -UseBasicParsing | iex` (Windows one-line entry)
  - from `elai install` subcommand (re-entry from already-installed CLI for repair / re-run)
  - from `.github/workflows/release.yml` on tag push (build pipeline that produces the assets the
    one-liners fetch)

stub_ban:
  - NO `# TODO` shell bodies; NO `$URL_HERE` placeholders; NO `exit 0  # smoke not implemented`
  - blake3 hashes in CHECKSUMS.blake3 are real after Phase 1 ships, OR explicitly marked
    `TBD-RUNPOD-PENDING:Phase-1` while the build pipeline is in flight
  - PowerShell `Get-CimInstance` must NOT silently fall through to `Get-WmiObject` on
    PS 5.1 (refuse, surface structured error telling user to install PowerShell 7+)
  - install.sh tool-check failures use ToolchainMissing canonical pattern (gotchas.md:625-649),
    NOT raw stack traces or `exit 1` with no message
  - HF rate-limit responses are retried with backoff (max 3), NOT silently swallowed

observable_effect:
  - post-install `elai doctor` returns exit 0 on Win64 + Linux x86_64 reference machines
  - post-install `elai run "ok"` (cloud mode OR remote tunnel) returns a valid Anthropic
    content block matching the `/v1/messages` shape per REMOTE-SIDECAR.md:115-118
  - `which elai` (POSIX) / `where elai` (Windows) returns the canonical binary path
  - `~/.elai/install-state.json` shows all 8 steps in `done` state after a clean run

## RISK ENUMERATION (top 5, with mitigations)

R1: hand-rolled GH Actions matrix has higher maintenance cost than cargo-dist over time —
  mitigation: matrix is <=30 lines per OS entry, sample referenced from canonical rust GH
  Actions starter; cargo-dist adoption left open as B-46 follow-up if maintenance burden
  surfaces. Decision is reversible (one PR introduces cargo-dist later).

R2: user on Apple Silicon hits hard-error and bounces with no recourse — mitigation: error
  message is structured + actionable + links to REMOTE-SIDECAR.md tunnel pattern as the
  working alternative (cloud-mode via remote box on user's friend's GPU). The hard-error is
  NOT a dead end; it routes to a documented working path.

R3: blake3 manifest drift between release artifact and installer — mitigation: Phase 1
  release.yml generates CHECKSUMS.blake3 atomically with the binary build step + uploads
  it as part of the same tagged release; installer fetches the tag's blake3 manifest in
  the same atomic step as the binary. No two-source-of-truth window.

R4: curl-pipe-bash MITM injection — mitigation: HTTPS-only fetch (TLS pinned to GH host),
  blake3 verify before exec, documented `curl | tee install.sh; review; bash install.sh`
  alternative in the plan body (and in the published install.sh banner), GH Releases as
  canonical source signed by GH attestations (sigstore-style).

R5: SearXNG native venv breaks on user systems where system-python 3.13 is default and
  3.12 is not available — mitigation: installer probes for `python3.12` specifically +
  surfaces structured ToolchainMissing error pointing to
  https://python.org/downloads/release/python-31210/ (verified in PRE_GATE). Does NOT
  silently fall back to 3.13 (which lacks flash-attn wheel on Windows per server.py FA2
  detection logic; SDPA fallback path is the answer but it's an opt-in not a silent default).

## Reads (mandatory before coding, plan-level)

- workflow/remote-setup/REMOTE-SIDECAR.md (full file — canonical sidecar boot + SearXNG topology)
- workflow/remote-setup/install-sidecar.sh (full file — 8-step idempotent pattern to mirror)
- workflow/remote-setup/setup-remote.ps1 (full file — PowerShell installer log/idempotent shape)
- .elai_cc/crates/providers/src/vram_tier.rs (full file — VRAM tier thresholds + override env)
- .elai_cc/crates/cli/src/doctor.rs (full file — DiagnosticCheck shape, DoctorReport::has_failures)
- .elai_cc/crates/cli/src/args.rs lines 1730-1760 + 5560-5580 (doctor subcommand parse pattern)
- .elai_cc/crates/cli/src/commands.rs lines 260-290 + 1560-1580 (SlashCommand dispatch pattern)
- .elai_cc/scripts/stage_onnx_models.py (full file — existing recipes + size guards)
- .elai_cc/scripts/install_dev_tools.sh (full file — ToolchainMissing pattern in production)
- workflow/memory-bank/gotchas.md lines 625-649 (ToolchainMissing canonical structured-error pattern)
- .github/workflows/archive-verify.yml (existing matrix shape — reference for new release.yml)

## Tasks

### Phase 1 — Release pipeline + binary build matrix (Session 1)

Spec references: workflow/remote-setup/REMOTE-SIDECAR.md (sidecar dep set for asset bundling),
context-package BLAST RADIUS "WARN no existing GH Actions release pipeline".

Reads (mandatory):
- .github/workflows/archive-verify.yml (existing matrix YAML shape)
- workflow/memory-bank/gotchas.md anti-slopsquatting section (URL verify before HTTP fetch)
- PRE_GATE rows 1-2 of plan-level table.

WIRING_CONTRACT:
  produces: .github/workflows/release.yml, scripts/install/CHECKSUMS.blake3 (populated by
    workflow run on tag push), GH attestations attached to each release asset.
  consumes: cargo build --release per matrix entry, blake3 1.8 workspace dep (already pinned),
    actions/attest-build-provenance @ pinned tag (anti-stale: pin SHA + verify exists).
  reachable: tag-push trigger on `v*.*.*`, manual workflow_dispatch for backfills.
  stub_ban: NO `runs-on: ubuntu-latest` without a pinned ubuntu version (use ubuntu-24.04);
    NO `actions/checkout@v4` without SHA pin verified at PRE_GATE; matrix BLOCKED entries for
    aarch64 / macOS arm64 fail loudly per Q2.
  observable_effect: tag-push produces 4 artifacts (win-x64, linux-x64) + CHECKSUMS.blake3 +
    attestations attached; 2 BLOCKED matrix entries report `unsupported per Q2 — see E-51`.

PRE_GATE: PRE_GATE rows 1-2 verified; actions/attest-build-provenance verified via
  `gh api repos/actions/attest-build-provenance` (anti-slopsquatting, NOT trained URL).

TDD:
- workflow_dispatch on a feature branch produces a draft release with both win-x64 + linux-x64
  binaries + CHECKSUMS.blake3 + attestations attached (smoke before tag-push exercised).
- intentional matrix entry attempting `runs-on: ubuntu-latest` with target=aarch64 FAILS with
  the structured Q2-error string (verified via grep over the workflow run log).

DETERMINISTIC_CHECK:
```bash
# 1. release.yml exists at .github/workflows/release.yml
test -f .github/workflows/release.yml
# 2. matrix includes win-x64 + linux-x64; aarch64 / macos-arm64 explicitly BLOCKED per Q2
rg "windows-2022|ubuntu-24.04" .github/workflows/release.yml
rg "Q2-BLOCKED: aarch64 / macOS arm64" .github/workflows/release.yml
# 3. blake3 manifest authoring step present
rg "blake3 .* > CHECKSUMS.blake3" .github/workflows/release.yml
# 4. GH attestations step present
rg "actions/attest-build-provenance" .github/workflows/release.yml
```

ADVERSARIAL:
- tag without manifest: release.yml MUST fail the publish step if CHECKSUMS.blake3 is empty
  or contains TBD-RUNPOD-PENDING rows at tag-push time.
- manifest hash drift: re-running the build on the same tag MUST produce identical blake3
  values (deterministic build verified via two consecutive workflow_dispatch runs).
- matrix entry timeout: each matrix entry has a 30-min timeout; on timeout the partial
  artifact is NOT uploaded (manifest atomicity).

ACCEPTANCE:
- [ ] release.yml drives 2 supported targets (win-x64, linux-x64) + 2 BLOCKED targets that
      fail-fast with Q2 structured error
- [ ] CHECKSUMS.blake3 populated atomically with binary upload
- [ ] GH attestations attached to each released asset
- [ ] Smoke workflow_dispatch on feature branch produces draft release end-to-end

### Phase 2 — Detection layer (OS / arch / GPU / VRAM tier) (Session 1-2)

Spec references: .elai_cc/crates/providers/src/vram_tier.rs:7-92 (VRAM thresholds +
  ELAI_VRAM_OVERRIDE env shape), REMOTE-SIDECAR.md (WSL2 nvidia-smi path).

Reads (mandatory):
- .elai_cc/crates/providers/src/vram_tier.rs (constants: VRAM_THRESHOLD_48GB=40GiB,
  _24GB=20GiB, _16GB=14GiB, _12GB=10GiB; below = Vram8Gb)
- workflow/remote-setup/install-sidecar.sh lines 1-50 (env-probe + nvidia-smi probe pattern)
- workflow/remote-setup/setup-remote.ps1 (Get-CimInstance probe pattern)

WIRING_CONTRACT:
  produces: scripts/install/elai_install_lib.sh `detect_os`, `detect_arch`, `detect_gpu`,
    `detect_vram_tier`; scripts/install/elai_install_lib.ps1 equivalents.
  consumes: `uname -s` + `uname -m` (POSIX), `nvidia-smi` (with WSL2 fallback path
    `/usr/lib/wsl/lib/nvidia-smi` per install-sidecar.sh:13), Get-CimInstance
    `Win32_OperatingSystem` + `Win32_VideoController` (PowerShell 7+).
  reachable: from install.sh / install.ps1 top-level dispatch.
  stub_ban: NO `uname` without quoting output; NO PowerShell 5.1 fallback (refuse with
    structured error pointing to PowerShell 7 install).
  observable_effect: `detect_vram_tier` echoes one of `8|12|16|24|48|cloud`, matching
    `elai_providers::vram_tier::VramTier` Display impl.

PRE_GATE: confirm `nvidia-smi` candidate paths via existing install-sidecar.sh source (line 13);
  confirm Get-CimInstance Win32_VideoController is PowerShell-7+ stable (WebSearch verify).

TDD:
- POSIX: `bash -c 'source elai_install_lib.sh; detect_arch'` on x86_64 returns `x86_64`;
  on simulated aarch64 (via env override) returns `aarch64` AND install.sh exits 1 with
  Q2 structured error before any HTTP fetch.
- Windows: install.ps1 invoked under PowerShell 5.1 surfaces structured error
  `PowerShell 7+ required` and exits 1.
- VRAM probe returns the same tier label the running CLI's `elai_providers::vram_tier::
  detect_vram_tier` would for the same machine.

DETERMINISTIC_CHECK:
```bash
# 1. Lib files exist
test -f scripts/install/elai_install_lib.sh && test -f scripts/install/elai_install_lib.ps1
# 2. 4 detection functions in each lib
for fn in detect_os detect_arch detect_gpu detect_vram_tier; do
  rg -q "^${fn}\(\)" scripts/install/elai_install_lib.sh || echo "MISSING: $fn (posix)"
done
rg "function detect_os|function detect_arch|function detect_gpu|function detect_vram_tier" scripts/install/elai_install_lib.ps1 | wc -l  # expected: 4
# 3. WSL2 nvidia-smi fallback path
rg "/usr/lib/wsl/lib/nvidia-smi" scripts/install/elai_install_lib.sh
# 4. VRAM tier outputs gated to the 6 known values
rg 'echo "(8|12|16|24|48|cloud)"' scripts/install/elai_install_lib.sh
```

ADVERSARIAL:
- no GPU (CPU-only machine): `detect_vram_tier` returns `cloud` (matches
  `VramTier::Cloud` per vram_tier.rs:50); installer proceeds in cloud-only mode.
- WSL2 with native + wsl nvidia-smi both present: prefer wsl path (install-sidecar.sh:13
  precedent).
- PowerShell 5.1: structured error, NOT a silent fallback to `Get-WmiObject` (which has
  different field names).
- Apple Silicon (Darwin arm64): hard-error per Q2; verified by ADVERSARIAL test that
  spoofs `uname -m=arm64; uname -s=Darwin` and confirms exit 1 with Q2 error text.

ACCEPTANCE:
- [ ] 4 detection functions present in both libs
- [ ] aarch64 / macOS arm64 hard-error verified (matches Q2 RESOLVED text verbatim)
- [ ] VRAM tier output matches detect_vram_tier (Rust) for same machine (cross-check test)

### Phase 3 — Binary fetch + blake3 verify + partial-resume (Session 2)

Spec references: PRE_GATE row 1 (GH release host), gotchas.md:625-649 (ToolchainMissing).

Reads (mandatory):
- workflow/remote-setup/install-sidecar.sh lines 50-130 (cu124-then-cu121 fallback pattern)
- .elai_cc/crates/providers/src/vram_tier.rs:64-92 (fallback chain shape — mirror for install)

WIRING_CONTRACT:
  produces: scripts/install/elai_install_lib.sh `fetch_binary`, `verify_blake3`,
    `mark_state(step, status)`; ~/.elai/install-state.json schema.
  consumes: curl (POSIX) / Invoke-WebRequest (Windows), blake3 CLI (or python -m blake3
    fallback verified at PRE_GATE), GH Releases asset URL pattern.
  reachable: install.sh / install.ps1 step 3.
  stub_ban: NO `curl ... | bash` re-entry inside install.sh (single bootstrap, no second
    hop); NO blake3 skip if manifest fetch failed (refuse + structured error).
  observable_effect: ~/.elai/install-state.json after step 3 shows step=fetch_binary
    status=done plus blake3_verified=true; re-run skips the step idempotently.

PRE_GATE: PLACEHOLDER-RESOLVED-AT-RELEASE for org slug (PRE_GATE row 1); blake3 CLI
  presence probed and a python -m blake3 fallback path confirmed.

TDD:
- network drop mid-download: ~/.elai/install-state.json shows step=fetch_binary
  status=failed; re-run resumes from this step (NOT from step 1).
- blake3 mismatch: installer refuses to install the binary, prints structured error
  identifying the manifest expected hash vs computed hash, exits 1.
- partial chunk corruption: re-verify on re-run catches the mismatch and refuses again
  (no silent overwrite of a bad binary).

DETERMINISTIC_CHECK:
```bash
# 1. State machine functions
rg "^fetch_binary\(\)|^verify_blake3\(\)|^mark_state\(\)" scripts/install/elai_install_lib.sh | wc -l
# expected: 3
# 2. install-state.json schema documented
rg "install-state.json" scripts/install/elai_install_lib.sh
# 3. blake3 verify is gated BEFORE chmod +x / execution
rg -A 5 "verify_blake3" scripts/install/install.sh | grep -q "chmod \+x"
# expected: chmod after verify line
# 4. HTTPS-only enforced (no http://)
! rg "http://github.com" scripts/install/install.sh
```

ADVERSARIAL:
- MITM swap of binary post-fetch: blake3 mismatch → refuse + structured error.
- curl-pipe-bash injection: HTTPS-only + manifest verify before exec mitigates; document
  `curl -fsSL <url> | tee install.sh; review; bash install.sh` alternative in install.sh
  banner.
- partial-resume corruption: re-running after kill -9 mid-fetch produces same blake3
  verdict on the next attempt (no leaked tmp file consumed without re-verify).

ACCEPTANCE:
- [ ] ~/.elai/install-state.json drives idempotent resume (verified by kill -9 + re-run)
- [ ] blake3 verify gates execution; mismatch refuses with structured error
- [ ] HTTPS-only enforced; documented `tee install.sh; review; bash` alternative present

### Phase 4 — Model staging (extends stage_onnx_models.py with blake3) (Session 2-3)

Spec references: .elai_cc/scripts/stage_onnx_models.py:22-72 (existing recipes), CLAUDE.md
  Anti-Stale-Training (blake3 mandatory, NOT blake2/sha2), Q6 RESOLVED (eager by default).

Reads (mandatory):
- .elai_cc/scripts/stage_onnx_models.py (full file)
- workflow/remote-setup/REMOTE-SIDECAR.md sidecar model selection (3B vs 7B per VRAM tier)
- PRE_GATE rows 2-3 of plan-level table (HF repo URLs).

WIRING_CONTRACT:
  produces: extended .elai_cc/scripts/stage_onnx_models.py with `verify_blake3()` step +
    CHECKSUMS.blake3 manifest authoring at end of run; `--skip-models` flag wiring.
  consumes: HF_HOME env (do NOT duplicate models if cache already present), HF rate-limit
    backoff, detected VRAM tier from Phase 2 (selects 3B vs 7B vs no local model).
  reachable: from install.sh / install.ps1 step 4; from `elai install --skip-models` to
    skip; from `elai install --models` to re-stage post-install.
  stub_ban: NO download without size guard preserved (MIN_ONNX_BYTES=1MB,
    MIN_TOK_BYTES=1kB per stage_onnx_models.py current); blake3 mismatch refuses with
    structured error, NEVER overwrites silently.
  observable_effect: .elai_cc/models/ contains the recipes' outputs; CHECKSUMS.blake3 in
    scripts/install/ matches; HF_HOME cache reused if already populated.

PRE_GATE: HF repos verified per PRE_GATE rows 2-3; `huggingface_hub` Python lib version
  pinned + verified per anti-slopsquatting.

TDD:
- on 24GB GPU (Vram24Gb tier): stages the 7B model + all 4 ONNX recipes; blake3 manifest
  populated with non-TBD hashes.
- on 8GB GPU (Vram8Gb tier): stages 3B model + ONNX recipes; emits a WARN diagnostic for
  the 7B being skipped per VRAM constraint.
- `--skip-models`: stage_onnx_models.py is NOT invoked; install-state.json shows
  step=stage_models status=skipped.
- HF rate-limit 429: backoff (max 3 retries with exponential backoff); structured error on
  exhaustion, NOT silent failure.

DETERMINISTIC_CHECK:
```bash
# 1. blake3 verification added to stage_onnx_models.py
rg "def verify_blake3|blake3.hash_file|hashlib.* blake3" .elai_cc/scripts/stage_onnx_models.py
# 2. CHECKSUMS.blake3 author step present
rg "CHECKSUMS.blake3" .elai_cc/scripts/stage_onnx_models.py
# 3. --skip-models flag in install.sh / install.ps1
rg -- "--skip-models" scripts/install/install.sh scripts/install/install.ps1
# 4. HF_HOME cache reuse honored
rg "HF_HOME" .elai_cc/scripts/stage_onnx_models.py
```

ADVERSARIAL:
- HF 429: 3 retries, then structured error pointing to HF status page.
- partial download (network drop): re-run resumes (blake3 verify of partial file fails,
  refuses + re-fetches).
- cache already populated: no-op (blake3 verify succeeds on existing files; skip).
- wrong VRAM tier (e.g., 3B selected on 24GB GPU because user overrode): WARN, suggest
  re-run with `elai install --reset-config` or env `ELAI_VRAM_OVERRIDE` reset.

ACCEPTANCE:
- [ ] blake3 verify gates every recipe; mismatch refuses
- [ ] HF_HOME cache reuse confirmed (no double-download on re-run)
- [ ] `--skip-models` flag works end-to-end (install completes, /task surfaces graceful
      "models not staged" message — handled in Phase 7 INTEGRATION)

### Phase 5 — SearXNG bootstrap (native Python venv, port 8889) (Session 3)

Spec references: REMOTE-SIDECAR.md:41-60 (native venv canonical), Q3 RESOLVED (Docker NOT
  in B-43 MVP), gotchas.md:625-649 (ToolchainMissing pattern).

Reads (mandatory):
- workflow/remote-setup/install-sidecar.sh lines 60-100 (venv + pip baseline pattern)
- workflow/remote-setup/REMOTE-SIDECAR.md lines 41-60 (SearXNG start command + port 8889)
- workflow/memory-bank/gotchas.md lines 625-649 (ToolchainMissing structured error)
- PRE_GATE rows 4-5 of plan-level table.

WIRING_CONTRACT:
  produces: scripts/install/elai_install_lib.sh `bootstrap_searxng()` + `_probe_python312()` +
    `_probe_port_free()`; ~/.elai/searxng.pid (PID file mirroring sidecar.pid pattern).
  consumes: python3.12 binary, searxng git repo, ss / Get-NetTCPConnection (port probe).
  reachable: install.sh / install.ps1 step 5.
  stub_ban: NO Docker fallback (Q3 RESOLVED — single canonical path); port conflict surfaces
    ToolchainMissing-style structured error, NOT a stack trace; Python 3.12 missing surfaces
    structured error pointing to PRE_GATE row 5.
  observable_effect: ~/.elai/venv-searxng/ populated; SearXNG NOT auto-started (LOCK-5
    hardware-heavy rule — searxng start is the user's opt-in `~/start-searxng.sh` per
    REMOTE-SIDECAR.md); ~/.elai/searxng.pid absent until user starts.

PRE_GATE: PRE_GATE rows 4-5 verified (searxng git ls-remote, python.org Python 3.12.10).

TDD:
- port 8889 already in use: structured error message includes the conflicting process
  identifier (pid), exits 1 with ToolchainMissing-shaped output.
- Python 3.12 missing: structured error points to python.org/downloads/release/python-31210/.
- venv corruption mid-install: re-run refuses, prompts user to `rm -rf ~/.elai/venv-searxng`
  + re-run. NEVER silently overwrites.

DETERMINISTIC_CHECK:
```bash
# 1. Single canonical SearXNG path (no Docker branch)
! rg "docker run.*searxng" scripts/install/
# 2. Port 8889 enforced (matches REMOTE-SIDECAR.md canonical)
rg "8889" scripts/install/elai_install_lib.sh
# 3. ToolchainMissing pattern adopted
rg "ToolchainMissing" scripts/install/elai_install_lib.sh
# 4. Python 3.12 probe (NOT Python 3 generic)
rg "python3.12" scripts/install/elai_install_lib.sh
```

ADVERSARIAL:
- port 8889 in use by another process: structured error names the conflicting pid +
  suggests `kill <pid>` or installer flag `--searxng-port <alt>` (future, not B-43).
- Python 3.12 missing: structured error pointing to python.org Python 3.12.10 download.
- venv corruption (e.g., partial pip install crash): refuse + suggest cleanup; never
  silently auto-rm.

ACCEPTANCE:
- [ ] SearXNG venv created idempotently; re-run is no-op
- [ ] port conflict surfaces ToolchainMissing structured error
- [ ] Python 3.12 absent surfaces structured error pointing to PRE_GATE row 5

### Phase 6 — Config generation (elai_cc.toml + credentials scaffold) (Session 3-4)

Spec references: .elai_cc/crates/providers/src/vram_tier.rs:7-50 (tier injection), B-37
  RuntimeBackend default mapping per detected hardware.

Reads (mandatory):
- .elai_cc/elai_cc.toml current shape (for reference, not full read — grep the [runtime]
  and [vram] sections only)
- B-37 plan §8 PerRoleRuntime default mapping (designer/executor/auditor backends)

WIRING_CONTRACT:
  produces: scripts/install/elai_cc.toml.template; scripts/install/elai_install_lib.sh
    `render_config()`; ~/.elai/credentials.json scaffold (`{}` with mode 0600 / NTFS ACL).
  consumes: detected VRAM tier from Phase 2; default RuntimeBackend per hardware (Sidecar
    on NVIDIA, AnthropicApi on cloud-only).
  reachable: install.sh / install.ps1 step 6.
  stub_ban: NO overwrite of existing elai_cc.toml without diff-and-prompt (user opt-in
    `--reset-config` required); NO overwrite of credentials.json if it contains keys;
    NO writing `[runtime] backend = ""` (empty string — refuse + structured error).
  observable_effect: rendered elai_cc.toml contains correct [vram] tier and [runtime]
    backend per detected hardware; credentials.json exists with chmod 600.

PRE_GATE: B-37 RuntimeBackend symbol confirmed in providers crate (`rg
  "enum RuntimeBackend" .elai_cc/crates/providers/src`).

TDD:
- on first install (no existing elai_cc.toml): renders fresh template with detected tier.
- on re-run with existing elai_cc.toml present: diff prompt; user must opt-in to overwrite.
- on re-run with credentials.json containing API key: NEVER overwrites; preserves intact.

DETERMINISTIC_CHECK:
```bash
# 1. Template file exists
test -f scripts/install/elai_cc.toml.template
# 2. render_config function present
rg "^render_config\(\)" scripts/install/elai_install_lib.sh
# 3. credentials.json scaffold with 0600 mode
rg "chmod 600.*credentials.json" scripts/install/elai_install_lib.sh
# 4. existing-file diff-and-prompt (NOT silent overwrite)
rg "diff.*elai_cc.toml" scripts/install/elai_install_lib.sh
```

ADVERSARIAL:
- existing elai_cc.toml present: diff prompt; without `--reset-config` flag, preserve.
- existing credentials.json with `anthropic_api_key`: preserve; do not touch.
- wrong tier in existing config (e.g., user has Vram24Gb but new probe says Vram12Gb):
  WARN, suggest `elai install --reset-config` or env override.

ACCEPTANCE:
- [ ] Fresh-install renders correct elai_cc.toml + credentials.json scaffold
- [ ] Re-run preserves existing config unless user opts in
- [ ] credentials.json mode is 0600 (POSIX) or NTFS-ACL-restricted (Windows)

### Phase 7 — CLI integration + doctor wiring + final smoke + INTEGRATION (Session 4-5)

Spec references: .elai_cc/crates/cli/src/doctor.rs:65-83 + 265-284 (DiagnosticCheck shape +
  DoctorReport::has_failures), context-package INTERFACE MAP doctor.rs extension surface.

Reads (mandatory):
- .elai_cc/crates/cli/src/doctor.rs (full file — for new check_* functions)
- .elai_cc/crates/cli/src/args.rs (subcommand parse pattern for "doctor" — clone for "install")
- .elai_cc/crates/cli/src/commands.rs (SlashCommand dispatch + help_topic Discovery group)

WIRING_CONTRACT:
  produces: crates/cli/src/install.rs (NEW — `HostInstaller` struct + re-entry from
    `elai install`); crates/cli/src/doctor.rs (EXTENDED — `check_runtime_assets`,
    `check_sidecar_health`, `check_vram_tier`, `check_model_staging`);
    crates/cli/src/args.rs (EXTENDED — `CliAction::Install` variant + parser);
    crates/cli/src/commands.rs (EXTENDED — "install" entry in Discovery & debugging).
  consumes: existing `DiagnosticCheck`, `DiagnosticLevel`, `DoctorReport`; existing
    `run_doctor` entrypoint; existing `parse_doctor_subcommand_args` pattern.
  reachable: from `elai install` CLI; from `elai doctor` post-install smoke.
  stub_ban: NO `unimplemented!()` in install.rs; NO `unwrap()` (per CLAUDE.md HARD RULE);
    NEW DiagnosticCheck functions return real DiagnosticCheck (NOT mock Ok); the install
    subcommand surface must not silently no-op when invoked twice (idempotent + report
    skipped steps).
  observable_effect: `elai install` from already-installed CLI re-runs the 8-step state
    machine idempotently; `elai doctor` exits 0 after successful install on supported OS;
    final printed banner says `run \`elai sidecar boot --local\` to start the local model,
    or set ANTHROPIC_BASE_URL to remote tunnel` per LOCK-5.

PRE_GATE: confirm crates/cli/src/install.rs is not pre-existing (Glob); confirm
  `crates/core::bootstrap::BootstrapPlan` is the unrelated CLI-startup struct (do NOT touch).

TDD:
- `elai install` on fresh machine runs all 8 steps end-to-end; install-state.json shows
  all `done`; doctor exits 0.
- `elai install` second invocation: idempotent (all 8 steps `done` from cache; doctor still
  exits 0).
- `elai install --skip-models` then `elai run /task "make plant ID app"`: graceful error
  message `models not staged, run elai install --models` (NOT a panic).
- doctor exits 0 BUT `elai run "ok"` fails (e.g., sidecar boot failed): INTEGRATION smoke
  catches and surfaces a Fail-level doctor diagnostic on next doctor invocation.

DETERMINISTIC_CHECK:
```bash
# 1. New install.rs file exists in crates/cli/src/
test -f .elai_cc/crates/cli/src/install.rs
# 2. HostInstaller struct (NOT BootstrapPlan)
rg "struct HostInstaller" .elai_cc/crates/cli/src/install.rs
! rg "struct BootstrapPlan" .elai_cc/crates/cli/src/install.rs
# 3. Four new doctor checks added
for fn in check_runtime_assets check_sidecar_health check_vram_tier check_model_staging; do
  rg "fn ${fn}" .elai_cc/crates/cli/src/doctor.rs || echo "MISSING: $fn"
done
# 4. CliAction::Install variant in args.rs
rg "CliAction::Install" .elai_cc/crates/cli/src/args.rs
# 5. "install" registered in Discovery & debugging command group
rg "install" .elai_cc/crates/cli/src/commands.rs | grep -i "discovery\|install"
# 6. cargo check workspace clean (auditor runs this, NOT main LLM mid-implementation)
# (deferred to auditor; recorded here as the only check that needs the build to succeed)
```

INTEGRATION (Phase 7 only):
- End-to-end matrix smoke on Win64 + Linux x86_64 reference machines:
  1. fresh-VM provision; curl-pipe-bash (POSIX) / iwr-pipe-iex (Windows) one-liner
  2. install completes in <=12 min wall-clock (eager model stage on Vram12Gb tier)
  3. `elai doctor` exits 0
  4. `elai run "ok"` returns valid Anthropic content block (cloud-mode against
     ANTHROPIC_BASE_URL=remote-tunnel per CLAUDE.md PATHs)
  5. `elai install` second invocation idempotent (no-op all 8 steps)
- Cloud-only-mode dry-run (no models staged): `elai install --skip-models` + `elai run
  "ok"` against remote tunnel; expect success.
- aarch64 / Apple Silicon adversarial smoke: install.sh on simulated arch returns exit 1
  with Q2 structured error verbatim (grep over stderr).

ADVERSARIAL:
- doctor exits 0 but elai run fails: caught by INTEGRATION end-to-end smoke; failure
  surfaces as Fail diagnostic on next doctor invocation (no silent green).
- `elai install` twice in a row: idempotent (covered by TDD + DETERMINISTIC_CHECK).
- `elai install --skip-models` then `elai run /task`: graceful error message ("models not
  staged, run `elai install --models`"), NOT panic.

ACCEPTANCE:
- [ ] crates/cli/src/install.rs ships HostInstaller struct + re-entry
- [ ] doctor.rs extended with 4 new checks composing into DoctorReport::has_failures
- [ ] args.rs + commands.rs surface `elai install` subcommand
- [ ] End-to-end smoke on Win64 + Linux x86_64 passes (INTEGRATION block)
- [ ] aarch64 / Darwin arm64 hard-error verified at INTEGRATION

## EXIT_CRITERIA

- [ ] All 7 phases audited PASS (auditor only, NOT main LLM — per orchestrate-protocol.md H-3)
- [ ] CHECKSUMS.blake3 populated with REAL hashes (no TBD-RUNPOD-PENDING rows in shipped
      manifest; Phase 1 populates these at tag-push time)
- [ ] `elai doctor` exits 0 on Win64 + Linux x86_64 reference machines
- [ ] aarch64 + macOS arm64 paths exit 1 with the Q2 RESOLVED structured error verbatim
      (verified by ADVERSARIAL test at Phase 2 + INTEGRATION re-verification at Phase 7)
- [ ] ONE commit in maintainer voice (lowercase i, no em-dashes, no Co-Authored-By trailer,
      no robotic prefix, no per-phase commits) per LOCK-7 + commit-cadence.md
- [ ] NO new Rust deps added to Cargo.toml without explicit user confirmation in plan thread
- [ ] workflow/reasoning/B-43-decisions.md authored alongside this plan
- [ ] Plan markdown <=700 lines (current file <=700 lines verified by `wc -l`)
- [ ] Each phase block <=90 lines (verified by section-count)
- [ ] PLACEHOLDER-RESOLVED-AT-RELEASE rows resolved before SHIPPED status (real GH org slug;
      elai.dev domain registration — see B-43-decisions.md "Pending user confirmation")

## Files touched

- CREATE: scripts/install/install.sh (~120 LOC)
- CREATE: scripts/install/install.ps1 (~150 LOC)
- CREATE: scripts/install/elai_install_lib.sh (~280 LOC)
- CREATE: scripts/install/elai_install_lib.ps1 (~300 LOC)
- CREATE: scripts/install/CHECKSUMS.blake3 (~50 lines manifest, populated by Phase 1)
- CREATE: scripts/install/elai_cc.toml.template (~80 LOC)
- CREATE: .github/workflows/release.yml (~180 LOC, matrix + blake3 + attestations)
- CREATE: .elai_cc/crates/cli/src/install.rs (~350 LOC, HostInstaller + re-entry)
- CREATE: workflow/reasoning/B-43-decisions.md (~150 lines)
- MODIFY: .elai_cc/crates/cli/src/doctor.rs (+~120 LOC, 4 new checks)
- MODIFY: .elai_cc/crates/cli/src/args.rs (+~40 LOC, CliAction::Install)
- MODIFY: .elai_cc/crates/cli/src/commands.rs (+~30 LOC, install entry)
- MODIFY: .elai_cc/scripts/stage_onnx_models.py (+~80 LOC, blake3 verify + CHECKSUMS author)

## Next

B-44 (signed_releases — Apple notarization + Windows EV cert beyond GH attestations) or
B-45 (dashboard_installer — Tauri v2 bundles + per-OS desktop integration), whichever the
user prioritizes after B-43 SHIPPED.
