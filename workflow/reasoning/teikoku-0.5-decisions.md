TEIKOKU Phase 0.5 decision doc (controller, session s-de7e4c60, 2026-06-19)

WHAT
Final Phase-0 sub-phase: design-system-FIRST, then the smallest end-to-end ELAI slice made VISIBLE in a Tauri v2 + Svelte 5 + Vite shell built INSIDE the teikoku WSL distro. Cycle = context-compiler (DONE) -> executor-A (design system) -> executor-B (shell + slice) -> auditor (build + SIGNAL design pass + cross-screen consistency) -> controller Phase-0 commit.

KEY DECISIONS
- Q5 RESOLVED: canonical token file is oklch() derived from the §3 hex anchors NOW (not role-named hex deferred). Grounding: DESIGN-TEIKOKU §3 + handoff line 84 both mandate oklch(). Deterministic transform; verified visually at the design pass.
- Build order is design-system-first and non-negotiable (LAW §2 consistency mandate IS the point of 0.5): ONE tokens file -> atomic components built ONCE + a COMPONENT INDEX -> shell composes the index -> slice wiring. A literal hex / magic px in a component, or a re-implemented component, is an audit BLOCK at the cross-screen consistency check.
- Layout authority: where the mockup's fixed three-zone layout contradicts LAW §5 (icon-rail + collapsible panels + movable/resizable workspace panes), LAW wins. Mockup is visual reference only, internally inconsistent, never copied per-screen.
- Slice plumbing: conductor gate_runner.py has hardcoded F:\ paths and needs a git worktree + base_commit, so the slice uses run_smoke_test() or threads explicit WSL paths; tauri.conf rewritten bun-not-node; dev-server reparented via nohup setsid.
- One chat turn runs THROUGH ~/teikoku/base-cc/dist/cli.js; free-API graft via PER-SESSION ANTHROPIC_BASE_URL only, never a global ANTHROPIC_API_KEY (suppresses plan-OAuth).

EXPECTED TEST OUTCOME (audit gates)
- bun build / svelte-check / tauri build green inside the distro.
- The slice runs end-to-end: new-session -> one CC turn -> one autonomous task -> one off-agent gate -> event recorded -> a live dashboard TILE shows it.
- SIGNAL design pass: schema pass|fail|unknown (unknown=fail), evidence region mandatory, semantic critique vs the SIGNAL mockup (NOT pixel-diff, no golden render yet). Zero surviving slop tells.
- Cross-screen consistency: two surfaces share IDENTICAL tokens + components (no per-screen copies).

UNKNOWNS / RISKS
- @fontsource-variable/hanken-grotesk existence/version UNVERIFIED — executor anti-slopsquats at edit time; fallback = @font-face bundle the woff2 directly. JetBrains Mono same.
- conductor primitives may need light shims to run inside the distro (WSL paths). Keep shims honest, no stubs in the slice path.
- Tauri v2 + Svelte 5 + bun toolchain interplay inside a fresh distro (first Tauri build here) — top failure mode; mine dashboard-v2's pinned versions, do not invent.

TOOLING CONSTRAINTS (carry)
PowerShell->wsl for all distro ops (Bash + Write tools gated in EXECUTING). Author distro files via wsl heredoc / base64 / a .sh inside the distro. nohup setsid for the dev server (WSL reaps one-shot bg procs). Screenshots for the design pass via CDP harness pattern (deviceScaleFactor 1, fromSurface true, --font-render-hinting=none).
