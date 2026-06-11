//! G-3: deterministic runtime blast-radius gates.
//!
//! Two PURE, fail-closed verdict functions that the orchestrator wires at the
//! real production entrypoints:
//!
//!   * [`check_fs_target`] — given a shell-EXPANDED filesystem target (after the
//!     caller has resolved `~`/`$VAR`/glob, see [`expand_fs_token`]) and the
//!     per-run worktree root, returns [`BlastRadiusVerdict::Block`] with kind
//!     [`BlastRadiusKind::FsEscape`] when the canonicalised real path is NOT
//!     contained inside the canonical worktree root. This closes the
//!     "validate the un-expanded string, the shell expands `~`->$HOME after"
//!     hazard (the `rm -rf ~/x` / `rm -rf ../../outside` shape).
//!   * [`check_egress_host`] — rejects a host containing a NUL byte or any
//!     ASCII control char BEFORE any name resolution, then confirms the
//!     resolved IP set is a subset of the pinned allow-set; otherwise
//!     [`BlastRadiusKind::EgressDenied`] (the SOCKS5/null-byte egress bypass).
//!
//! CROSS-PLATFORM (Tier-1): every primitive here is `std::fs`/`std::net` only
//! and is the fail-closed done-gate on Windows + macOS + Linux. The kernel
//! backstop (Landlock / seccomp / veth, Linux-only) is a SEPARATE Tier-2 layer
//! that no-ops on non-Linux. This module makes NO claim of kernel-level
//! filesystem confinement or kernel egress restriction on any platform; it is
//! a userspace deterministic prefix/allow-set check. Landlock and macOS
//! Seatbelt are not used here (they are absent in-tree).
//!
//! FAIL-CLOSED on ambiguity: an un-canonicalisable target, an unresolvable
//! host, or a glob whose literal base could match outside the root all BLOCK.
//! No LLM classifier, no substring heuristic — verdict is pure prefix / set
//! membership (an LLM permission classifier collapses to ~81% false-negative
//! under adversarial scope-escalation, arXiv:2604.04978).
//!
//! Grounding: G-3-decisions.md (DD1/DD2/DD4); the existing cross-platform
//! canonicaliser pattern at `orchestrator::sandbox_policy::canonicalise_for_check`
//! (re-implemented self-contained here so `elai-safety` stays dependency-free
//! and gains no edge into `elai-orchestrator`).

use std::ffi::OsString;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};

/// The blocked-flow discriminator carried on the SSB row. Two arms: a
/// filesystem escape (the post-expansion target left the worktree root) and an
/// egress denial (the host was rejected or resolved outside the allow-set).
///
/// Mirrors the `orchestrator::ssb::BlastRadiusKind` wire enum; kept here as the
/// safety-crate-local source of truth so the pure gate has no orchestrator dep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadiusKind {
    /// A filesystem-mutating verb's expanded target resolved OUTSIDE the
    /// per-run worktree root (`rm -rf ~/x`, `mv x ../../outside`, a symlink or
    /// glob escaping the root).
    FsEscape,
    /// An outbound connect target was rejected: the host carried a NUL/control
    /// char, could not be resolved, or resolved to an IP outside the pinned
    /// allow-set.
    EgressDenied,
}

impl BlastRadiusKind {
    /// Stable snake_case wire label (matches the SSB `kind` field).
    pub fn label(self) -> &'static str {
        match self {
            Self::FsEscape => "fs_escape",
            Self::EgressDenied => "egress_denied",
        }
    }
}

/// Verdict of a blast-radius gate. `Block` carries the full PRE- and
/// POST-expansion command strings — the `#10077` forensics fix: the expanded
/// command (the one that would actually have run) was never logged before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlastRadiusVerdict {
    /// The operation is contained / the host is pinned. Proceed.
    Allow,
    /// The operation escapes the worktree root or the host is denied. Refuse
    /// fail-closed and emit a `blast_radius_blocked` SSB row.
    Block {
        /// Which boundary was crossed.
        kind: BlastRadiusKind,
        /// The original, un-expanded command/argv as the agent issued it.
        pre_expansion: String,
        /// The resolved/canonicalised target (FS) or rejected host+ip (egress)
        /// — what would actually have executed/connected.
        post_expansion: String,
    },
}

impl BlastRadiusVerdict {
    /// True when the verdict permits the operation.
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// blake3 hex-prefix length retained for the worktree-root identifier on the
/// SSB row (an identifier, not a secret; blake3 ONLY per CLAUDE.md HARD RULE).
pub const WORKTREE_ROOT_HASH_HEX_LEN: usize = 16;

/// blake3 hex-prefix of the canonical worktree root path string. Used as the
/// `worktree_root_blake3` field on the SSB row so a forensic reader can group
/// blocks by the run's root without storing the absolute path verbatim.
pub fn worktree_root_blake3(worktree_root: &Path) -> String {
    let canon = canonicalise_for_check(worktree_root);
    let hex = blake3::hash(canon.to_string_lossy().as_bytes()).to_hex();
    hex.as_str()[..WORKTREE_ROOT_HASH_HEX_LEN].to_owned()
}

// ---------------------------------------------------------------------------
// Filesystem post-expansion gate
// ---------------------------------------------------------------------------

/// Glob metacharacters that make a token's match-set unbounded. A token
/// containing any of these is treated by [`expand_fs_token`] as a glob: the
/// literal prefix BEFORE the first metachar is the only part we can canonicalise
/// soundly, so we check containment of that prefix and fail closed if it could
/// reach outside the root.
const GLOB_METACHARS: &[char] = &['*', '?', '[', ']', '{', '}'];

/// Outcome of Rust-side expansion of a single path-like token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandedToken {
    /// A concrete (post `~`/`$VAR`) path to canonicalise and prefix-check.
    Path(PathBuf),
    /// The token contained a glob metachar; only the literal base dir before
    /// the first metachar is known. Containment is checked on the base; if the
    /// base is at/inside the root the glob can only expand within the tree, so
    /// it is allowed — otherwise it is blocked (a glob that COULD match outside
    /// the root is refused, DD1 fail-closed-on-ambiguity).
    GlobBase(PathBuf),
}

/// Expand a single shell path token Rust-side: `~`/`~/...` -> `$HOME`,
/// `$VAR` / `${VAR}` -> the process env value, and detect glob metachars.
///
/// This is NOT a full shell parser (DD1 honest limit: an obfuscated command
/// using `eval`, command substitution, or base64-decode-then-exec can evade
/// token extraction; the Linux Tier-2 kernel backstop covers that on Linux,
/// Windows has no Tier-2). It handles the documented `#10077`/`#12637` shapes:
/// tilde, environment variables, and globs.
///
/// `home_dir` is injected (not read from env here) so the gate is deterministic
/// and testable; production passes `std::env::var_os("HOME")` /
/// `"USERPROFILE"`.
pub fn expand_fs_token(token: &str, home_dir: Option<&Path>) -> ExpandedToken {
    let expanded = expand_vars_and_tilde(token, home_dir);

    // Detect the first glob metachar; if present, keep only the literal base
    // directory before it (the longest leading run of full path components that
    // contain no metachar).
    if expanded.chars().any(|c| GLOB_METACHARS.contains(&c)) {
        let base = literal_glob_base(&expanded);
        return ExpandedToken::GlobBase(base);
    }
    ExpandedToken::Path(PathBuf::from(expanded))
}

/// Substitute `~`/`~/` (leading) with `home_dir`, and `$VAR` / `${VAR}` with the
/// process environment value (empty string when unset — an unset var that
/// collapses a path is caught by the containment check, not silently allowed).
fn expand_vars_and_tilde(token: &str, home_dir: Option<&Path>) -> String {
    // Tilde only expands at the very start of the token (shell semantics).
    let after_tilde: String = if token == "~" {
        home_dir
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|| "~".to_owned())
    } else if let Some(rest) = token.strip_prefix("~/") {
        match home_dir {
            Some(h) => {
                let mut s = h.to_string_lossy().into_owned();
                s.push(std::path::MAIN_SEPARATOR);
                s.push_str(rest);
                s
            }
            None => token.to_owned(),
        }
    } else {
        token.to_owned()
    };

    substitute_env_vars(&after_tilde)
}

/// Replace `$VAR` and `${VAR}` occurrences with the process env value. Unknown
/// vars expand to an empty string (matching `sh -lc` with `set -u` off, the
/// ELAI default). A literal `$` not followed by a name passes through.
fn substitute_env_vars(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                // ${VAR}
                if let Some(end) = s[i + 2..].find('}') {
                    let name = &s[i + 2..i + 2 + end];
                    if let Some(val) = std::env::var_os(name) {
                        out.push_str(&val.to_string_lossy());
                    }
                    i = i + 2 + end + 1;
                    continue;
                }
            } else {
                // $VAR — name is [A-Za-z_][A-Za-z0-9_]*
                let name_start = i + 1;
                let mut j = name_start;
                while j < bytes.len()
                    && (bytes[j] == b'_' || bytes[j].is_ascii_alphanumeric())
                    && !(j == name_start && bytes[j].is_ascii_digit())
                {
                    j += 1;
                }
                if j > name_start {
                    let name = &s[name_start..j];
                    if let Some(val) = std::env::var_os(name) {
                        out.push_str(&val.to_string_lossy());
                    }
                    i = j;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// The longest leading literal directory of a glob token (everything up to the
/// path separator that precedes the first glob metachar). `/home/u/*.rs` ->
/// `/home/u`; `../*` -> `..`; `*.rs` -> `` (current dir, treated as the root's
/// own dir by the caller which joins it against cwd before checking).
fn literal_glob_base(expanded: &str) -> PathBuf {
    let first_meta = expanded
        .char_indices()
        .find(|(_, c)| GLOB_METACHARS.contains(c))
        .map(|(idx, _)| idx)
        .unwrap_or(expanded.len());
    let literal = &expanded[..first_meta];
    // Trim back to the last separator so we keep only whole components.
    let cut = literal
        .rfind(['/', '\\'])
        .map(|idx| &literal[..idx])
        .unwrap_or("");
    PathBuf::from(cut)
}

/// THE FS gate. `expanded_target` is a single post-expansion path (or glob
/// base) to be written/deleted; `worktree_root` is the per-run containment
/// boundary; `pre_expansion` is the original argv string for the SSB row.
///
/// Returns [`BlastRadiusVerdict::Block`]`{ FsEscape }` when the canonicalised
/// real path is NOT contained inside the canonical worktree root, OR when the
/// path cannot be canonicalised at all (fail-closed). An in-root write returns
/// [`BlastRadiusVerdict::Allow`] (the false-positive guard — a legit write must
/// not be blocked).
pub fn check_fs_target(
    expanded_target: &Path,
    worktree_root: &Path,
    pre_expansion: &str,
) -> BlastRadiusVerdict {
    // 1) Reject NUL / control chars in the path FIRST (a NUL truncates the
    //    path at the syscall boundary — the path the kernel sees differs from
    //    the one we check, a classic confused-deputy).
    let target_str = expanded_target.to_string_lossy();
    if has_nul_or_control(&target_str) {
        return BlastRadiusVerdict::Block {
            kind: BlastRadiusKind::FsEscape,
            pre_expansion: pre_expansion.to_owned(),
            post_expansion: format!("<rejected: NUL/control char in path> {target_str}"),
        };
    }

    let canon_root = canonicalise_for_check(worktree_root);
    let canon_target = canonicalise_for_check(expanded_target);

    if path_starts_with(&canon_target, &canon_root) {
        BlastRadiusVerdict::Allow
    } else {
        BlastRadiusVerdict::Block {
            kind: BlastRadiusKind::FsEscape,
            pre_expansion: pre_expansion.to_owned(),
            post_expansion: canon_target.to_string_lossy().into_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Egress host gate
// ---------------------------------------------------------------------------

/// THE egress gate. Rejects a host carrying a NUL byte or any ASCII control
/// char BEFORE any resolution (the SOCKS5/null-byte bypass), then confirms that
/// EVERY resolved IP for `host` is present in the pinned `allowed` set. A host
/// that fails to resolve, resolves to zero addresses, or resolves to any IP
/// outside the set returns [`BlastRadiusVerdict::Block`]`{ EgressDenied }`
/// (fail-closed). `resolver` is injected for determinism in tests; production
/// passes [`resolve_host_std`].
pub fn check_egress_host<R>(
    host: &str,
    allowed: &[IpAddr],
    resolver: R,
) -> BlastRadiusVerdict
where
    R: Fn(&str) -> Vec<IpAddr>,
{
    // 1) NUL / control-char reject BEFORE resolve.
    if host.is_empty() || has_nul_or_control(host) {
        return BlastRadiusVerdict::Block {
            kind: BlastRadiusKind::EgressDenied,
            pre_expansion: host.to_owned(),
            post_expansion: format!("<rejected: NUL/control char or empty host> {host:?}"),
        };
    }

    // 2) A bare IP literal is checked directly against the allow-set (no DNS).
    if let Ok(ip) = host.parse::<IpAddr>() {
        return if allowed.contains(&ip) {
            BlastRadiusVerdict::Allow
        } else {
            BlastRadiusVerdict::Block {
                kind: BlastRadiusKind::EgressDenied,
                pre_expansion: host.to_owned(),
                post_expansion: format!("ip={ip} not in pinned allow-set"),
            }
        };
    }

    // 3) Resolve + pin: every resolved IP must be in the allow-set.
    let resolved = resolver(host);
    if resolved.is_empty() {
        return BlastRadiusVerdict::Block {
            kind: BlastRadiusKind::EgressDenied,
            pre_expansion: host.to_owned(),
            post_expansion: format!("host={host} did not resolve (fail-closed)"),
        };
    }
    if resolved.iter().all(|ip| allowed.contains(ip)) {
        BlastRadiusVerdict::Allow
    } else {
        let pinned: Vec<String> = resolved.iter().map(ToString::to_string).collect();
        BlastRadiusVerdict::Block {
            kind: BlastRadiusKind::EgressDenied,
            pre_expansion: host.to_owned(),
            post_expansion: format!("host={host} resolved {pinned:?} not all in allow-set"),
        }
    }
}

/// Default production resolver: `std::net` `ToSocketAddrs`. Returns an empty
/// Vec on any resolution failure (the caller treats empty as fail-closed
/// Block). Appends `:0` because `ToSocketAddrs` requires a port.
pub fn resolve_host_std(host: &str) -> Vec<IpAddr> {
    use std::net::ToSocketAddrs;
    match (host, 0u16).to_socket_addrs() {
        Ok(iter) => iter.map(|sa| sa.ip()).collect(),
        Err(_) => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Shared primitives (self-contained: no orchestrator dependency)
// ---------------------------------------------------------------------------

/// True when `s` contains a NUL byte or any ASCII control character. Control
/// chars in a host/path are never legitimate and are a confused-deputy / log-
/// injection vector.
fn has_nul_or_control(s: &str) -> bool {
    s.bytes().any(|b| b == 0 || b.is_ascii_control())
}

/// Best-effort canonicalisation, cross-platform. `std::fs::canonicalize`
/// requires the path to exist; for a not-yet-created target (the common case
/// for a write) we canonicalise the deepest existing ancestor and re-append the
/// missing tail, then strip the Windows verbatim prefix (`\\?\`) so containment
/// comparison is consistent regardless of existence. `..` and symlinks in the
/// existing portion are resolved by `std::fs::canonicalize`; `..` in the
/// non-existent tail is normalised component-wise. Mirrors the proven
/// `orchestrator::sandbox_policy::canonicalise_for_check` (re-implemented to
/// keep `elai-safety` free of an orchestrator edge).
fn canonicalise_for_check(path: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(path) {
        return strip_verbatim_prefix(c);
    }
    let mut existing: PathBuf = path.to_path_buf();
    let mut tail: Vec<OsString> = Vec::new();
    while !existing.as_os_str().is_empty() {
        if existing.exists() {
            break;
        }
        let leaf = existing.file_name().map(std::borrow::ToOwned::to_owned);
        let parent_opt = existing.parent().map(Path::to_path_buf);
        match parent_opt {
            Some(p) if p.as_os_str().is_empty() => break,
            Some(p) => {
                if let Some(name) = leaf {
                    tail.push(name);
                }
                existing = p;
            }
            None => break,
        }
    }
    let mut out = if let Ok(c) = std::fs::canonicalize(&existing) {
        strip_verbatim_prefix(c)
    } else {
        // Nothing exists — normalise component-wise (resolve `.`/`..` lexically).
        let mut acc = PathBuf::new();
        for comp in path.components() {
            match comp {
                Component::ParentDir => {
                    acc.pop();
                }
                Component::CurDir => {}
                other => acc.push(other.as_os_str()),
            }
        }
        return acc;
    };
    for name in tail.into_iter().rev() {
        // A `..` tail component must still pop (the ancestor existed but the
        // child does not yet): normalise lexically as we re-append.
        if name == OsString::from("..") {
            out.pop();
        } else if name != OsString::from(".") {
            out.push(name);
        }
    }
    out
}

/// Strip the Windows verbatim prefix (`\\?\`) so component-wise comparison
/// works between existing and non-existing paths under the same root. No-op on
/// non-Windows.
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.as_os_str().to_string_lossy().into_owned();
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
        path
    }
    #[cfg(not(windows))]
    {
        path
    }
}

/// Component-boundary prefix containment (so `/tmp/foo` does NOT contain
/// `/tmp/foobar`). Both paths must already be canonicalised.
fn path_starts_with(candidate: &Path, prefix: &Path) -> bool {
    let cand: Vec<_> = candidate.components().collect();
    let pref: Vec<_> = prefix.components().collect();
    if pref.len() > cand.len() {
        return false;
    }
    cand.iter().zip(pref.iter()).all(|(a, b)| a == b)
}

// ---------------------------------------------------------------------------
// Unit tests (pure-logic; the cross-crate runtime/wiring assertions live in
// crates/contracts/tests/g3_blast_radius_contract.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn kind_labels_are_stable_snake_case() {
        assert_eq!(BlastRadiusKind::FsEscape.label(), "fs_escape");
        assert_eq!(BlastRadiusKind::EgressDenied.label(), "egress_denied");
    }

    #[test]
    fn tilde_expands_to_home() {
        let home = PathBuf::from("/home/maintainer");
        let e = expand_fs_token("~/secret", Some(&home));
        assert_eq!(e, ExpandedToken::Path(PathBuf::from("/home/maintainer/secret")));
    }

    #[test]
    fn env_var_expands() {
        std::env::set_var("ELAI_TEST_BR_VAR", "/etc");
        let e = expand_fs_token("$ELAI_TEST_BR_VAR/passwd", None);
        assert_eq!(e, ExpandedToken::Path(PathBuf::from("/etc/passwd")));
        std::env::remove_var("ELAI_TEST_BR_VAR");
    }

    #[test]
    fn glob_keeps_literal_base() {
        let e = expand_fs_token("/home/maintainer/*.rs", None);
        assert_eq!(e, ExpandedToken::GlobBase(PathBuf::from("/home/maintainer")));
    }

    #[test]
    fn in_root_write_is_allowed_no_block() {
        // Use a real temp dir so canonicalize succeeds.
        let root = std::env::temp_dir();
        let target = root.join("elai_g3_inroot_file.txt");
        let v = check_fs_target(&target, &root, "echo > elai_g3_inroot_file.txt");
        assert!(v.is_allow(), "in-root write must be Allow (FP guard): {v:?}");
    }

    #[test]
    fn escape_above_root_is_blocked() {
        let root = std::env::temp_dir().join("elai_g3_subroot");
        // ../ escapes the subroot back into temp_dir.
        let target = root.join("..").join("elsewhere.txt");
        let v = check_fs_target(&target, &root, "rm -rf ../elsewhere.txt");
        match v {
            BlastRadiusVerdict::Block {
                kind: BlastRadiusKind::FsEscape,
                ..
            } => {}
            other => panic!("escape must Block FsEscape, got {other:?}"),
        }
    }

    #[test]
    fn nul_in_path_is_blocked() {
        let root = std::env::temp_dir();
        let target = PathBuf::from(format!("{}/a\u{0}b", root.display()));
        let v = check_fs_target(&target, &root, "touch a\\0b");
        assert!(!v.is_allow(), "NUL path must Block");
    }

    #[test]
    fn allowed_ip_host_is_allowed() {
        let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
        let v = check_egress_host("host.example", &pinned, |_| {
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))]
        });
        assert!(v.is_allow(), "pinned host must Allow (FP guard): {v:?}");
    }

    #[test]
    fn null_byte_host_is_blocked_before_resolve() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
        let resolved = AtomicBool::new(false);
        let v = check_egress_host("evil\u{0}.example", &pinned, |_| {
            resolved.store(true, Ordering::SeqCst);
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))]
        });
        assert!(!v.is_allow(), "null-byte host must Block");
        assert!(
            !resolved.load(Ordering::SeqCst),
            "must reject BEFORE resolve (fail-closed before connect)"
        );
    }

    #[test]
    fn unpinned_resolved_ip_is_blocked() {
        let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
        let v = check_egress_host("attacker.example", &pinned, |_| {
            vec![IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9))]
        });
        match v {
            BlastRadiusVerdict::Block {
                kind: BlastRadiusKind::EgressDenied,
                ..
            } => {}
            other => panic!("unpinned IP must Block EgressDenied, got {other:?}"),
        }
    }

    #[test]
    fn unresolvable_host_is_blocked_fail_closed() {
        let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
        let v = check_egress_host("nowhere.invalid", &pinned, |_| Vec::new());
        assert!(!v.is_allow(), "unresolvable host must Block (fail-closed)");
    }

    #[test]
    fn worktree_root_hash_is_blake3_prefix() {
        let root = std::env::temp_dir();
        let h = worktree_root_blake3(&root);
        assert_eq!(h.len(), WORKTREE_ROOT_HASH_HEX_LEN);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
