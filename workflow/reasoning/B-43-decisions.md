B-43 one_line_install — decisions doc
Date authored: 2026-05-29
Plan path: docs/thefinalglance/B-plan/B-43_one_line_install.md
Status: PLANNED
GRC budget: 8/1500 per topic="b43-plan-author" (set at plan-author time, single topic)

Predecessor plans: B-37 (RuntimeBackend dispatch — installer routes hardware to backend),
  B-42 (cosmos/ tier — installer mkdir's ~/.elai/cosmos/), C-28 (install_dev_tools.sh
  prior-art for ToolchainMissing + idempotent dev-tool install).
Successor plans (DEFERRED from B-43): B-44 (signed_releases — Apple notarization +
  Windows EV cert), B-45 (dashboard_installer — Tauri v2 .msi/.dmg/AppImage), B-46
  (package_manager_distribution — Homebrew tap + scoop + AUR + winget), separate
  plan auto-updater, E-51 (MLX adapter for Apple Silicon sidecar), separate plan
  Linux desktop integration (.desktop / .deb / .AppImage).

RESOLVED open questions (user-locked, baked into plan)

Q1 RESOLVED: hand-rolled GH Actions release matrix. NO cargo-dist. NO new Rust deps.
  Executor REC where user said "best option": matrix entry per OS bounded to <=30 lines,
  sample cribbed from canonical rust GH Actions starter; cargo-dist adoption left open
  as B-46 follow-up if maintenance burden surfaces in production. Reversible.

Q2 RESOLVED: HARD ERROR for Apple Silicon (macOS arm64) and Linux aarch64. install.sh /
  install.ps1 detect arch, print the structured error verbatim (`ELAI local sidecar
  unsupported on aarch64 / Apple Silicon until E-51 MLX adapter ships. Use remote sidecar
  tunnel (see workflow/remote-setup/REMOTE-SIDECAR.md) or wait for E-51.`) and exit 1.
  No silent cloud-only fallback (hides the platform gap). Error message routes user to a
  documented working path (remote tunnel), so this is NOT a dead-end UX.

Q3 RESOLVED: native Python venv default for SearXNG (port 8889 per REMOTE-SIDECAR.md
  canonical). Docker is NOT supported in B-43 MVP. The `--searxng-docker` flag is dropped
  entirely. scripts/start-searxng.sh's Docker pattern is orthogonal and stays out of B-43
  install flow. Single canonical path avoids dual-topology drift.

Q4 RESOLVED: blake3 CHECKSUMS.blake3 manifest + GH attestations (sigstore-style).
  Apple notarization + Windows EV cert deferred to B-44_signed_releases. GH attestations
  give baseline supply-chain verifiability without per-OS code-signing infrastructure cost.

Q5 RESOLVED: pure shell scripts. install.sh (POSIX, `bash -euo pipefail`) +
  install.ps1 (PowerShell 7+, `$ErrorActionPreference="Stop"`). NO second-hop bootstrap
  binary. Matches rustup / ollama / uv / bun canonical pattern. Minimizes attack surface
  (one transport, one verifier, no chained downloads) and reduces installer maintenance.

Q6 RESOLVED: install-time eager model staging by default. Flag `--skip-models` for CI /
  corp-firewalled / Docker-image-baking workflows. First /task is fast (models already
  on disk). Eager default reflects "happy-path fresh install on personal box" being the
  dominant case; opt-out flag handles the orthogonal use cases without making them
  default-pain.

LOCKED architectural decisions (do not re-litigate in B-43 thread)

LOCK-1: BootstrapPlan name collision. crates/core/src/bootstrap.rs's `BootstrapPlan`
  remains the CLI-startup bootstrap (CliEntry / FastPathVersion / etc) per
  E48_FOUR_PRE_EXISTING_NAME_COLLISIONS taxonomy. New host-installer artifact in
  crates/cli/src/install.rs is named `HostInstaller`. Wrap-not-rename — leave bootstrap.rs
  untouched. Disambiguates by file boundary, not by renaming the older symbol.

LOCK-2: sidecar boot command set is LOCKED from REMOTE-SIDECAR.md:69-76. Installer reuses
  the exact form (`ELAI_FORCE_SDPA=1 SEARXNG_URL=http://127.0.0.1:8889 python server.py
  --model huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated --port 8100 --no-dola
  --turboquant-bits 0 --no-steering --load-in-4bit`). No fork. No drift. Any change to
  the boot command updates REMOTE-SIDECAR.md first, then propagates to installer.

LOCK-3: POSIX installer mirrors install-sidecar.sh's 8-step idempotent pattern.
  Re-runnable. Partial-resume state at `~/.elai/install-state.json` (step + status +
  blake3_verified flag per step). Mirroring an already-running canonical pattern reduces
  the risk surface vs reinventing.

LOCK-4: Tauri dashboard install is DEFERRED to B-45_dashboard_installer. B-43 ships CLI
  + sidecar runtime + doctor only. The dashboard binary is NOT downloaded or staged in
  B-43. Scope discipline — B-43 already touches CI matrix + 2 shells + 4 doctor checks +
  CLI subcommand; adding Tauri bundles would double the matrix maintenance cost.

LOCK-5: sidecar is NOT auto-booted post-install (HARDWARE-HEAVY rule from CLAUDE.md).
  Install ends with `elai doctor` exit 0 + printed next-step hint: ``run `elai sidecar
  boot --local` to start the local model, or set ANTHROPIC_BASE_URL to remote tunnel``.
  Explicit user opt-in avoids silently spinning up a GPU process that the user expected
  to start when they were ready.

LOCK-6: anti-slopsquatting. Every URL in the plan has a PRE_GATE verification row
  (curl -sfI / git ls-remote / WebFetch). Every HF repo ID has a verify command. No
  "from training memory" URLs. Slopsquatting protection is a HARD RULE per CLAUDE.md.

LOCK-7: one commit at end of plan in maintainer voice per .claude/rules/commit-cadence.md
  (lowercase i, no em-dashes, no Co-Authored-By trailer, no robotic prefix). No
  per-phase commits. The commit lands ONCE after Phase 7 ships and EXIT_CRITERIA is
  satisfied. Multi-commit chunking is treated as a P0 incident per commit-cadence.md.

GRC log

Topic: "b43-plan-author"
Budget: 8/1500 (Tier HEAVY classification per planning-workflow SKILL.md; 7 phases,
  3-5 sessions, 12-18 new files)
Reasoning steps in this authoring pass: 0 (plan structure mirrored verbatim from B-37 /
  B-40 / B-41 / B-42 canonical shape; user-locked decisions resolved upstream by
  prompt-engineer; no contested tradeoffs surfaced in authoring).

Pending user confirmation (resolve BEFORE SHIPPED status)

(a) RESOLVED 2026-05-29: GH org slug locked to `maintainer/elai` per user. Asset URLs in
    install.sh + install.ps1 + release.yml now point at
    `https://example.invalid/archived-repositories/elai/releases/download/<tag>/<asset>`. The repo itself
    must exist before the first tag-push CI run (create at github.com/new under the
    maintainer account when ready to publish).

(b) RESOLVED 2026-05-29: elai.dev domain NOT registered. Canonical one-liner uses the
    GH raw URL fallback documented in the install.sh banner:
      curl -fsSL https://example.invalid/archived-repositories/elai/main/scripts/install/install.sh | bash
    Domain registration deferred to a future patch; not on the B-43 critical path.

Open follow-up plan IDs reserved (do not allocate to other work)

B-44 — signed_releases — Apple notarization + Windows EV cert beyond GH attestations.
B-45 — dashboard_installer — Tauri v2 .msi / .dmg / AppImage bundles + per-OS desktop
       integration (.desktop file Linux, Start Menu shortcut Windows, dock icon macOS).
B-46 — package_manager_distribution — Homebrew tap + scoop manifest + AUR PKGBUILD +
       winget manifest. Considered as overflow path if hand-rolled GH matrix proves
       higher-maintenance than expected.

Cross-references

- CLAUDE.md "Anti-Stale-Training" (blake3 mandatory NOT blake2/sha2)
- CLAUDE.md "Anti-Slopsquatting" (verify every URL before HTTP fetch)
- CLAUDE.md "Remote sidecar (canonical)" (REMOTE-SIDECAR.md boot command — LOCK-2 source)
- .claude/rules/commit-cadence.md (LOCK-7 source — one commit, maintainer voice)
- .claude/rules/orchestrate-protocol.md (H-3 / H-4 — auditor + memory-writer mandatory
  at end of each phase, NOT main LLM running tests / writing memory)
- workflow/memory-bank/gotchas.md:625-649 (ToolchainMissing canonical pattern — referenced
  by Phase 2 / Phase 5 / Phase 6)
- workflow/sessions/s-74468/context-package.md (sole upstream context for this plan)
