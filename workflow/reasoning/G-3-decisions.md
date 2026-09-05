G-3 decisions — Deterministic runtime blast-radius gates
session s-56068, 2026-06-04

WHAT
Two deterministic, fail-closed gates emitting a blast_radius_blocked SSB row with the FULL pre- AND post-expansion command:
(a) FS post-EXPANSION write/delete gate — canonicalize ~/glob/symlink/.. BEFORE the verb runs, fail closed if the resolved path is outside the worktree root.
(b) Host-side egress IP-pin — resolve+pin the allowed IP, reject hostnames with null bytes / control chars, before connect.

GROUND TRUTH (context-compiler + 2 explorers, s-56068, fresh elai.map):
- ELAI SHELL-EXPANDS: bash.rs:234/270 hands the whole command to `sh -lc <cmd>` (Unix) / `cmd /C` (Windows); core/src/sandbox.rs:301-303 also `sh -lc`. So ~/$VAR/glob/symlink expand AFTER Rust validation = the #10077 shape. The argv-exec path (dispatcher.rs:1013) is a SEPARATE compile-only path, not where agent bash lands.
- Production FS entrypoint: run_bash (crates/tools/src/lib.rs:2067); gate inserts AFTER safety_filter::check_bash_safety, BEFORE execute_bash, fail-closed.
- Cross-platform canonicalization EXISTS: sandbox_policy.rs:436 canonicalise_for_check (symlink + .. resolution, non-existent-path fallback, Windows \\?\ strip) + :508 path_starts_with (component-boundary prefix). REUSE these.
- bash_validation.rs (validate_paths / command_targets_outside_workspace) is WIRE-DARK (zero prod callers), substring-based, fail-OPEN (~/ and ../ only Warn). Do NOT wire as-is.
- Egress: NO host-side enforced connect-time gate exists. researcher_net.rs is Linux-only (#[cfg(target_os="linux")]), is_active:false off-Linux, fails OPEN (log-only). seccomp opens AF_INET wholesale. NO null-byte/control-char host reject anywhere.
- Baseline SCHEMA_VERSION 210 (ssb.rs:2928), TOTAL_VARIANTS 487 (action_type_lock.rs:567) -> 211/488.

KEY DECISIONS

DD1 FS GATE shape. Hook in run_bash (tools/src/lib.rs:2067), after check_bash_safety, before execute_bash, fail-closed. For FS-mutating commands (compose with E-30's verb detection check_argv at hook_dispatcher.rs:959 — gate the VERB there; G-3 gates the PATH here; do NOT re-implement E-30), extract path-like tokens, Rust-side expand ~ -> $HOME + known $VARs + resolve globs (globset — already a workspace dep per CLAUDE.md; NO new dep), then canonicalise_for_check (resolves symlink/..) and path_starts_with against the worktree root. Any resolved target outside the root -> BLOCK + blast_radius_blocked{FsEscape} carrying BOTH the original argv AND the canonicalized/expanded path (the #10077 forensics fix — the command was never logged before).

DD2 EGRESS GATE shape (NEW cross-platform Rust). Gate at the ORCHESTRATOR BOUNDARY where the run Arc<Ssb> is in scope (grounding_client / research path), NOT inside the leaf web_search crate (fetcher.rs/client.rs have NO Ssb dep — the recurring SSB-ctor wire-dark trap). Before connect: (1) reject host containing null bytes / control chars; (2) resolve hostname -> pin the resolved IP -> allow connect only to the pinned IP (or the allowed_hosts set). Reject -> BLOCK + blast_radius_blocked{EgressDenied} carrying the rejected host + the resolved/pinned IP. Compose with (do not duplicate) researcher_net.rs allowed_hosts as the policy source.

DD3 SSB. NEW blast_radius_blocked Pattern-2 single-variant, 2-arm Kind = FsEscape | EgressDenied. Payload: original argv + canonicalized path (FS) OR rejected host + resolved IP (egress). Bump SCHEMA 210->211 + TOTAL_VARIANTS 487->488 in lockstep; regen action_type_lock.golden (manual/codegen path). Mirror the just-shipped RunArtifactEmitted/TaintFlow cascade. FS arm emits via production_agent_rpc's Option<&Arc<Ssb>> (G-1 mirror, emit at the run_bash gate's caller if Ssb reaches there; else thread it — confirm at edit time). Egress arm emits via the orchestrator-boundary run Ssb. STANDING CHECK: trace each emit to a production path that holds the live Ssb.

DD4 CROSS-PLATFORM HONESTY (GD-5, W-10 tiered). Tier-1 = the deterministic canonicalize+prefix FS check + IP-pin + null-byte reject = CROSS-PLATFORM (Windows + Linux); it is the fail-closed done-gate. Tier-2 = the Linux-only kernel backstop (Landlock/seccomp/veth at core/src/sandbox.rs:277, researcher_net.rs) = returns None / no-op on Windows+macOS. The decision doc + any user-facing text MUST NOT claim kernel FS confinement or kernel egress restriction on Windows/macOS — on those platforms only the deterministic Rust gate applies. Landlock/macOS-Seatbelt are ABSENT in-tree; do not claim them. Scope non-Linux kernel parity as a separate follow-up.

DD5 DRAWER LABEL FIX: G-3 adds its own 'blast-radius' (or chosen key) label to DRAWER_LABELS in Drawer.svelte AND back-fills the MISSING 'rra-viewer' label (G-2's dashboard pair left it out — confirmed by the egress explorer; it would fail svelte-check). frontend_pair: a blast-radius/egress drawer mirroring G-4 McpRugPullDrawer / G-2 RraViewerPanel + Tauri command + types.ts DTO + DrawerHost branch.

DD6 WORKTREE ROOT: there is no central worktree_root() helper; scratch_dir is a PathBuf field (agent_pool.rs:78). The gate resolves the allowed root from the run's cwd/session root available at the run_bash dispatch point. Windows canonicalize yields \\?\ — normalize before the prefix check (canonicalise_for_check already strips it).

EXPECTED TEST OUTCOME
Deterministic fixtures (cross-platform): (1) `rm -rf $X` where $X expands to ~/ (or a symlink/.. escaping the worktree) -> the resolved path is outside root -> BLOCKED before execute_bash + a blast_radius_blocked{FsEscape} row carrying the original argv AND the canonicalized ~/ path; (2) a write INSIDE the worktree -> ALLOWED, no row (FP guard); (3) a fetch to a host containing a null byte / control char -> BLOCKED before connect + blast_radius_blocked{EgressDenied} carrying the rejected host; (4) a fetch to an allowed host -> resolved+pinned, ALLOWED, no row (FP guard). The block row's pre+post-expansion fields re-derive from the journal under ELAI_SEED (GD-2).

GD-4 / GD-5 / W-5
GD-4 FP bound: the in-process FP guards (in-worktree write allowed; allowed-host fetch allowed) are the proxies; record the bound, do NOT advertise default-ON before >=3 real runs. GD-5: the Tier-1/Tier-2 split + the non-Linux honesty above. W-5: a live `elai run` driving a real rm-rf/egress fixture = Tier-2 separately tracked; the deterministic in-process fixture is the Tier-1 fail-closed gate.

UNKNOWNS / RISKS
- Token extraction is not a full shell parser; an obfuscated command (eval, base64-decode-then-exec, command substitution) can evade FS token extraction -> honest limit; Linux Tier-2 is the backstop, Windows has none (document).
- Whether the live Arc<Ssb> reaches run_bash directly (tools crate) or must be threaded from production_agent_rpc — confirm at edit time; if threading >5 files, prefer emitting from the nearest production caller that holds the Ssb (mirror G-1's enforce_taint_sink_gate which already runs at the dispatch boundary with Option<&Arc<Ssb>>).
- globset glob expansion vs the shell's glob semantics may differ at the edges — the gate errs fail-closed (if a glob COULD match outside root, block).
