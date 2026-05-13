//! Phase B1 — Tier C Sandbox policy layer.
//!
//! This module is the policy-level sibling of `orchestrator::sandbox`
//! (the per-process `AgentSandbox`/AppContainer abstraction). The two
//! abstractions are deliberately separate:
//!
//!   * `sandbox.rs`        — process isolation runtime (Windows
//!                           AppContainer, Linux user-namespace).
//!   * `sandbox_policy.rs` — language-level allow/deny gate consulted by
//!                           the in-process tool dispatcher BEFORE any
//!                           bash / file / network operation runs.
//!
//! The dispatcher (see `crates/tools/src/dispatcher.rs`) holds an
//! `Option<Arc<dyn SandboxAdapter>>`. When the field is `None` the
//! enforcement is a documented no-op (preserves backward compatibility
//! with callers that have not yet wired the adapter). When it is
//! `Some`, every dispatch entry point calls
//! [`SandboxAdapter::check_policy`] BEFORE the underlying work runs and
//! maps a `SandboxDenied` verdict into
//! `ToolDispatchError::Denied { reason }`.
//!
//! Default policy (see [`SandboxPolicy::default_for_cwd`]):
//!   * `profile = WritableProjectDir`
//!   * `network_allow_list = []`     (network access denied by default)
//!   * `fs_writable_paths = [cwd]`   (writes outside cwd are denied)
//!   * Destructive bash commands (rm -rf /, dd if=, mkfs.*, etc.) are
//!     denied regardless of profile.
//!
//! The `PassthroughSandbox` impl below is intentionally a policy-only
//! adapter — it does NOT spawn an isolation runtime. "Passthrough"
//! refers to the runtime layer; the policy layer always runs.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

// ---------------------------------------------------------------------------
// Profile + policy data
// ---------------------------------------------------------------------------

/// Coarse-grained sandbox profile. The dispatcher consults the profile
/// before checking the finer-grained fields on [`SandboxPolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxProfile {
    /// Filesystem is read-only; every `FileWrite` is denied.
    ReadOnlyFs,
    /// `fs_writable_paths` is consulted; writes elsewhere are denied.
    WritableProjectDir,
}

/// Full policy record consulted by [`SandboxAdapter::check_policy`].
///
/// Fields are public so callers can build a policy struct-literal in
/// tests / setup code without going through a builder. Inside a running
/// dispatcher the struct is wrapped in an `Arc<dyn SandboxAdapter>` and
/// shared across both local-provider and cloud-provider dispatches —
/// the policy layer is intentionally provider-source-agnostic.
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub profile: SandboxProfile,
    /// Hostnames the dispatcher is allowed to reach. Empty = deny all.
    pub network_allow_list: Vec<String>,
    /// Directories the dispatcher is allowed to write under (a write is
    /// permitted iff the canonicalised path lies inside one of these,
    /// per a prefix match on the path components).
    pub fs_writable_paths: Vec<PathBuf>,
}

impl SandboxPolicy {
    /// Default "WritableProjectDir under cwd, empty network allow-list,
    /// deny destructive commands" policy. Used by
    /// [`PassthroughSandbox::with_default_policy`].
    ///
    /// On systems where the cwd cannot be read (extremely rare — only
    /// happens when the process has been chdir-ed into a deleted dir),
    /// the policy falls back to `std::env::temp_dir()` which is always
    /// present.
    pub fn default_for_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
        Self {
            profile: SandboxProfile::WritableProjectDir,
            network_allow_list: Vec::new(),
            fs_writable_paths: vec![cwd],
        }
    }
}

// ---------------------------------------------------------------------------
// Op + verdict
// ---------------------------------------------------------------------------

/// One operation the dispatcher is about to perform. Borrowed form so
/// the dispatcher can pass references without allocating on the hot
/// path. Variants intentionally cover the three classes of side-effects
/// the in-process dispatcher can produce.
#[derive(Debug)]
pub enum SandboxOp<'a> {
    /// Arbitrary process / shell invocation. `cmd` is the program; `args`
    /// are the literal argv tail (no shell parsing).
    BashCommand { cmd: &'a str, args: &'a [String] },
    /// Filesystem write under `path`. Reads are intentionally NOT
    /// modeled — the dispatcher's read paths are governed by the
    /// `ReadOnlyFs` profile only.
    FileWrite { path: &'a Path },
    /// Outbound network access. `host` is the destination hostname
    /// (port is irrelevant for the allow-list check).
    NetworkAccess { host: &'a str },
}

/// Denial verdict returned from [`SandboxAdapter::check_policy`]. The
/// dispatcher converts this into `ToolDispatchError::Denied { reason }`.
#[derive(Debug, Clone)]
pub struct SandboxDenied {
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Object-safe trait implemented by every concrete sandbox adapter. The
/// dispatcher stores an `Option<Arc<dyn SandboxAdapter>>`; the `Send +
/// Sync` bounds are required for that storage to be shared between the
/// CLI runtime and the dashboard's tokio runtime.
pub trait SandboxAdapter: Send + Sync {
    /// Consult the policy. `Ok(())` permits the operation; `Err(...)`
    /// surfaces a structured denial reason. Implementations must be
    /// pure (no I/O) so the dispatcher hot path stays predictable.
    fn check_policy(&self, op: &SandboxOp) -> Result<(), SandboxDenied>;
}

// ---------------------------------------------------------------------------
// Passthrough impl — enforces the policy in pure Rust without spawning
// an isolation runtime.
// ---------------------------------------------------------------------------

/// Policy-only adapter. Use this when the caller wants the policy
/// guards (destructive-command denial, network allow-list, write
/// allow-list) but does not want to spin up a real isolation runtime.
///
/// Holds the policy by value so the adapter can be cloned cheaply and
/// shared across dispatcher instances via `Arc<dyn SandboxAdapter>`.
#[derive(Debug, Clone)]
pub struct PassthroughSandbox {
    policy: SandboxPolicy,
}

impl PassthroughSandbox {
    /// Wrap an explicit policy.
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    /// Convenience: wrap [`SandboxPolicy::default_for_cwd`].
    pub fn with_default_policy() -> Self {
        Self::new(SandboxPolicy::default_for_cwd())
    }

    /// Borrow the configured policy.
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }
}

impl SandboxAdapter for PassthroughSandbox {
    fn check_policy(&self, op: &SandboxOp) -> Result<(), SandboxDenied> {
        match *op {
            SandboxOp::BashCommand { cmd, args } => check_bash(cmd, args),
            SandboxOp::FileWrite { path } => check_file_write(&self.policy, path),
            SandboxOp::NetworkAccess { host } => check_network(&self.policy, host),
        }
    }
}

// ---------------------------------------------------------------------------
// Policy check helpers
// ---------------------------------------------------------------------------

/// Catalogue of destructive-command patterns the sandbox refuses. Each
/// variant maps to a focused `matches_*` check in [`check_bash`] so the
/// detector is a small enum of named intents rather than a regex blob.
/// Auditor BLOCK-1 (B1 review): the previous regex-based matcher could
/// be evaded by splitting the `rm` flag across multiple argv tokens
/// (e.g. `argv=["rm","-r","-f","/"]` joins to `"rm -r -f /"`, which
/// failed the `[rf]*r[rf]*f[rf]*` pattern because `r` and `f` never
/// co-occurred in the same `-rf` token). The argv-level checks below
/// inspect every short-flag token's char set and every long-flag arg
/// independently, so flag splitting no longer evades the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DestructivePattern {
    /// `rm -rf /` (and any flag-split / long-flag / double-slash
    /// variant pointing at the filesystem root).
    DeleteRoot,
    /// `dd if=...` raw device overwrite.
    DiskOverwrite,
    /// `mkfs.<fstype> ...` filesystem creation.
    MakeFilesystem,
    /// Classic bash fork bomb `:(){ :|:& };:`.
    ForkBomb,
    /// Shell redirection into `/dev/sd[a-z]` (still string-shaped — the
    /// dispatcher passes redirections inside `cmd` literally because
    /// SandboxOp::BashCommand documents "no shell parsing").
    DeviceRedirect,
}

/// Regex set retained only for the string-shaped patterns
/// ([`DestructivePattern::ForkBomb`] and
/// [`DestructivePattern::DeviceRedirect`]) — these are not argv-shaped
/// (a fork bomb is a single token, a redirection is shell syntax) so
/// argv-level inspection cannot help. Compiled once per process via
/// `OnceLock`.
fn string_shaped_patterns() -> &'static [(DestructivePattern, Regex)] {
    static PATTERNS: OnceLock<Vec<(DestructivePattern, Regex)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let raw: &[(DestructivePattern, &str)] = &[
            (DestructivePattern::ForkBomb, r"(?i):\(\)\{\s*:\|:\s*&\s*\};:"),
            (DestructivePattern::DeviceRedirect, r"(?i)>\s*/dev/sd[a-z]"),
        ];
        raw.iter()
            .filter_map(|(p, r)| Regex::new(r).ok().map(|rx| (*p, rx)))
            .collect::<Vec<_>>()
    })
}

/// Tokenise a single-string command into argv tokens. Used when the
/// caller passes the entire shell line in `cmd` (e.g. `cmd="rm -rf /"`
/// with empty `args`) — we recover the argv so the argv-level checks
/// fire uniformly regardless of how the caller chose to split. Quoting
/// is NOT honoured: SandboxOp::BashCommand documents "no shell
/// parsing", so a token like `'rm -rf /'` would stay as one token. The
/// detector treats that as a single `cmd` and consequently does not
/// hit the `rm` argv path — which is the correct behaviour (a literal
/// quoted token is not executable as a destructive `rm`).
fn whitespace_tokens(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// True if a positional argument refers to the filesystem root. We
/// accept `"/"` exactly, plus pure-slash strings (`"//"`, `"///"`,
/// ...) — these all canonicalise to `/` and would have the same blast
/// radius. Any further path component (`"/etc"`, `"//etc"`) is NOT
/// matched here: that is a write-to-sensitive-path concern, not a
/// nuke-the-root concern, and is handled by FileWrite policy.
fn arg_is_root(arg: &str) -> bool {
    !arg.is_empty() && arg.chars().all(|c| c == '/')
}

/// Collect every short-flag char from a list of tokens. A short-flag
/// token starts with a single `-` (not `--`) and the chars after the
/// dash are its flag set (`-rf` -> `{'r','f'}`, `-r` -> `{'r'}`,
/// `-rfv` -> `{'r','f','v'}`). Non-flag tokens are ignored. The
/// combined char set is what the auditor flagged: previously only the
/// chars inside a single token were checked, so `["-r","-f"]` left `r`
/// and `f` in separate sets and the AND-of-rf test missed.
fn collect_short_flag_chars<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<char> {
    let mut out = Vec::new();
    for tok in tokens {
        if tok.len() >= 2 && tok.starts_with('-') && !tok.starts_with("--") {
            for c in tok[1..].chars() {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// True if any token in `tokens` is `--<flag>` or `--<flag>=<value>`.
fn has_long_flag<'a>(mut tokens: impl Iterator<Item = &'a str>, flag: &str) -> bool {
    let needle = format!("--{flag}");
    let needle_eq = format!("{needle}=");
    tokens.any(|t| t == needle || t.starts_with(&needle_eq))
}

fn matches_delete_root<'a>(program: &str, all_tokens: &[&'a str]) -> bool {
    if program != "rm" {
        return false;
    }
    // tokens after the program name = the argv tail.
    let tail: Vec<&str> = all_tokens.iter().skip(1).copied().collect();
    let short = collect_short_flag_chars(tail.iter().copied());
    let recursive =
        short.contains(&'r') || short.contains(&'R') || has_long_flag(tail.iter().copied(), "recursive");
    let force = short.contains(&'f') || has_long_flag(tail.iter().copied(), "force");
    if !(recursive && force) {
        return false;
    }
    // Any positional (non-flag) arg points at the filesystem root.
    tail.iter()
        .any(|t| !t.starts_with('-') && arg_is_root(t))
}

fn matches_disk_overwrite<'a>(program: &str, all_tokens: &[&'a str]) -> bool {
    if program != "dd" {
        return false;
    }
    // `dd if=...` (or `dd of=/dev/...`) — flag the read-source form
    // explicitly because that is what the brief lists. We match any
    // arg of shape `if=<value>` regardless of value: the legitimate
    // uses of `dd if=` inside the sandbox are zero (we do not image
    // disks from the agent path).
    all_tokens.iter().skip(1).any(|t| t.starts_with("if="))
}

fn matches_make_filesystem<'a>(program: &str, _all_tokens: &[&'a str]) -> bool {
    // `mkfs.<fstype>` is a family of programs (`mkfs.ext4`,
    // `mkfs.btrfs`, ...). Lower-cased compare: the previous regex was
    // case-insensitive, so we preserve that for behavioural parity.
    let lowered = program.to_ascii_lowercase();
    lowered.starts_with("mkfs.") && lowered.len() > "mkfs.".len()
}

fn check_bash(cmd: &str, args: &[String]) -> Result<(), SandboxDenied> {
    // Build the unified token stream the argv-level checks see. The
    // caller is allowed to pass the destructive payload either as a
    // pre-split argv (`cmd="rm"`, `args=["-r","-f","/"]`) or as a
    // single shell-style string (`cmd="rm -rf /"`, `args=[]`). We
    // first tokenise `cmd` to recover any embedded arguments, then
    // append `args` verbatim. The first token after tokenisation is
    // the program name; the rest are the argv tail.
    let cmd_tokens = whitespace_tokens(cmd);
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut all_tokens: Vec<&str> = Vec::with_capacity(cmd_tokens.len() + arg_refs.len());
    all_tokens.extend(cmd_tokens.iter().copied());
    all_tokens.extend(arg_refs.iter().copied());

    // Compose the inspection string for string-shaped patterns and
    // for the denial-reason payload. Mirrors the previous behaviour.
    let mut full = String::with_capacity(
        cmd.len() + args.iter().map(|a| a.len() + 1).sum::<usize>(),
    );
    full.push_str(cmd);
    for a in args {
        full.push(' ');
        full.push_str(a);
    }

    let program = all_tokens.first().copied().unwrap_or("");

    let argv_hits: &[(DestructivePattern, fn(&str, &[&str]) -> bool)] = &[
        (DestructivePattern::DeleteRoot, matches_delete_root),
        (DestructivePattern::DiskOverwrite, matches_disk_overwrite),
        (DestructivePattern::MakeFilesystem, matches_make_filesystem),
    ];
    for (pat, check) in argv_hits {
        if check(program, &all_tokens) {
            return Err(SandboxDenied {
                reason: format!(
                    "destructive command denied by policy [{pat:?}]: {full}"
                ),
            });
        }
    }

    for (pat, rx) in string_shaped_patterns() {
        if rx.is_match(&full) {
            return Err(SandboxDenied {
                reason: format!(
                    "destructive command denied by policy [{pat:?}]: {full}"
                ),
            });
        }
    }
    Ok(())
}

fn check_file_write(policy: &SandboxPolicy, path: &Path) -> Result<(), SandboxDenied> {
    if matches!(policy.profile, SandboxProfile::ReadOnlyFs) {
        return Err(SandboxDenied {
            reason: format!(
                "ReadOnlyFs profile denies write to {}",
                path.display()
            ),
        });
    }
    if policy.fs_writable_paths.is_empty() {
        return Err(SandboxDenied {
            reason: format!(
                "no fs_writable_paths configured; denying write to {}",
                path.display()
            ),
        });
    }
    // Empty path is the "no path information" placeholder used by some
    // dispatch entry points (e.g. dispatch_classify_failure). Treat it
    // as a no-op: the call carries no filesystem effect, so the policy
    // layer permits it. Callers that want a stricter posture can install
    // a custom SandboxAdapter that rejects empty paths.
    if path.as_os_str().is_empty() {
        return Ok(());
    }
    let candidate = canonicalise_for_check(path);
    for allowed in &policy.fs_writable_paths {
        let allowed_canon = canonicalise_for_check(allowed);
        if path_starts_with(&candidate, &allowed_canon) {
            return Ok(());
        }
    }
    Err(SandboxDenied {
        reason: format!(
            "write to {} is outside fs_writable_paths",
            path.display()
        ),
    })
}

fn check_network(policy: &SandboxPolicy, host: &str) -> Result<(), SandboxDenied> {
    // WARN-3 (B1 audit): DNS hostnames are case-insensitive per RFC
    // 4343 §2, so the allow-list lookup must lower-case both sides.
    // The previous byte-exact compare let `EVIL.com` slip past an
    // allow-list configured with `evil.com` (and vice-versa for the
    // false-positive direction). Lower-casing is ASCII-only — IDN
    // (Punycode) labels are already ASCII at this layer, so
    // `to_ascii_lowercase` is the correct operator.
    let host_lc = host.to_ascii_lowercase();
    if policy
        .network_allow_list
        .iter()
        .any(|h| h.to_ascii_lowercase() == host_lc)
    {
        return Ok(());
    }
    Err(SandboxDenied {
        reason: format!("host {host} not in network_allow_list"),
    })
}

/// Best-effort canonicalisation. `std::fs::canonicalize` requires the
/// path to exist; for non-existent paths (the common case for
/// `FileWrite` of a yet-to-be-created file) we canonicalise the deepest
/// existing ancestor and re-append the missing components, then strip
/// the Windows verbatim prefix (`\\?\`) so prefix containment checks
/// compare consistently regardless of whether the path exists.
fn canonicalise_for_check(path: &Path) -> PathBuf {
    // Fast path — path exists.
    if let Ok(c) = std::fs::canonicalize(path) {
        return strip_verbatim_prefix(c);
    }
    // Walk up until we find an existing ancestor, then re-append the
    // tail. This keeps the prefix shape consistent with the writable
    // paths (which are canonicalised when they exist) and avoids the
    // Windows `\\?\` mismatch between existing-dir + non-existing-child.
    let mut existing: PathBuf = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    while !existing.as_os_str().is_empty() {
        if existing.exists() {
            break;
        }
        let leaf = existing.file_name().map(|n| n.to_owned());
        // Compute the parent without holding a borrow into `existing`
        // so the upcoming `existing = parent` reassignment compiles.
        let parent_opt = existing.parent().map(|p| p.to_path_buf());
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
        // Component-wise normalisation when nothing exists.
        let mut out = PathBuf::new();
        for comp in path.components() {
            match comp {
                std::path::Component::ParentDir => {
                    out.pop();
                }
                std::path::Component::CurDir => {}
                other => out.push(other.as_os_str()),
            }
        }
        return out;
    };
    for name in tail.into_iter().rev() {
        out.push(name);
    }
    out
}

/// Strip the Windows verbatim prefix (`\\?\`) from a canonicalised path
/// so component-wise comparison works between existing and
/// non-existing paths under the same root. No-op on non-Windows.
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

/// Prefix containment that respects path component boundaries (so
/// `/tmp/foo` does NOT contain `/tmp/foobar`).
fn path_starts_with(candidate: &Path, prefix: &Path) -> bool {
    let cand: Vec<_> = candidate.components().collect();
    let pref: Vec<_> = prefix.components().collect();
    if pref.len() > cand.len() {
        return false;
    }
    cand.iter().zip(pref.iter()).all(|(a, b)| a == b)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_uses_writable_project_dir_profile() {
        let p = SandboxPolicy::default_for_cwd();
        assert_eq!(p.profile, SandboxProfile::WritableProjectDir);
        assert!(p.network_allow_list.is_empty());
        assert_eq!(p.fs_writable_paths.len(), 1);
    }

    #[test]
    fn passthrough_denies_rm_rf_root() {
        let sb = PassthroughSandbox::with_default_policy();
        let cmd = "rm -rf /".to_string();
        let args: Vec<String> = Vec::new();
        let op = SandboxOp::BashCommand {
            cmd: &cmd,
            args: &args,
        };
        assert!(sb.check_policy(&op).is_err());
    }

    #[test]
    fn passthrough_denies_dd_if() {
        let sb = PassthroughSandbox::with_default_policy();
        let cmd = "dd if=/dev/zero of=/dev/sda".to_string();
        let args: Vec<String> = Vec::new();
        let op = SandboxOp::BashCommand {
            cmd: &cmd,
            args: &args,
        };
        assert!(sb.check_policy(&op).is_err());
    }

    #[test]
    fn passthrough_allows_ordinary_command() {
        let sb = PassthroughSandbox::with_default_policy();
        let cmd = "git".to_string();
        let args = vec!["status".to_string()];
        let op = SandboxOp::BashCommand {
            cmd: &cmd,
            args: &args,
        };
        assert!(sb.check_policy(&op).is_ok());
    }

    #[test]
    fn passthrough_denies_unlisted_host() {
        let sb = PassthroughSandbox::with_default_policy();
        let op = SandboxOp::NetworkAccess { host: "evil.com" };
        assert!(sb.check_policy(&op).is_err());
    }

    #[test]
    fn read_only_profile_denies_all_writes() {
        let policy = SandboxPolicy {
            profile: SandboxProfile::ReadOnlyFs,
            network_allow_list: Vec::new(),
            fs_writable_paths: vec![std::env::temp_dir()],
        };
        let sb = PassthroughSandbox::new(policy);
        let path = std::env::temp_dir().join("any.txt");
        let op = SandboxOp::FileWrite {
            path: path.as_path(),
        };
        assert!(sb.check_policy(&op).is_err());
    }

    #[test]
    fn empty_filewrite_path_is_treated_as_no_op() {
        // The dispatch_classify_failure hook fires with an empty path
        // because the operation has no filesystem effect. Document that
        // the policy layer permits it under WritableProjectDir.
        let policy = SandboxPolicy {
            profile: SandboxProfile::WritableProjectDir,
            network_allow_list: Vec::new(),
            fs_writable_paths: vec![std::env::temp_dir()],
        };
        let sb = PassthroughSandbox::new(policy);
        let empty = Path::new("");
        let op = SandboxOp::FileWrite { path: empty };
        assert!(sb.check_policy(&op).is_ok());
    }
}
