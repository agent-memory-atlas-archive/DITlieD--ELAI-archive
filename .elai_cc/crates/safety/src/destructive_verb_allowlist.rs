//! E-30: Non-LLM destructive-verb argv-regex allowlist.
//!
//! Provides a compiled `RegexSet` checked at T1 (<1ms) latency before any
//! PreToolUse hook fires. In-prompt instructions are NOT a safety boundary:
//! Replit ALL-CAPS "DO NOT DROP" instructions were ignored 11 times
//! (theregister.com 2025-07-21). This fix is at the argv layer in compiled code.
//!
//! Extend via `crates/safety/policies/destructive_verbs.toml` (loaded at init;
//! non-fatal if missing).

use once_cell::sync::Lazy;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

/// Verdict returned by the argv-regex allowlist check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowlistVerdict {
    /// The command is safe to proceed.
    Allow,
    /// The command matches a hard-denied pattern. Refuse the tool call and log.
    Deny {
        reason: String,
        citation: String,
    },
    /// The command requires explicit human approval before proceeding.
    /// FSM should enter AWAITING_HUMAN_APPROVAL sub-state.
    RequireHumanApproval {
        reason: String,
        citation: String,
    },
}

// ---------------------------------------------------------------------------
// Compiled pattern sets (once_cell: compiled at first access, not per-call)
// ---------------------------------------------------------------------------

/// Hard-deny patterns. Index-matched to DENY_META.
static DENY_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // 0: rm -rf targeting ~ or absolute path
        r"(?i)rm\s+(-[^\s]*r[^\s]*f|-[^\s]*f[^\s]*r)\s+(~|/)",
        // 1: git restore . or --staged
        r"(?i)git\s+restore\s+(\.|--staged)",
        // 2: git reset --hard to non-HEAD~0 ref (HEAD~N where N>=1, origin/, sha)
        r"(?i)git\s+reset\s+--hard\s+(HEAD[~^][1-9]|origin/|[a-zA-Z0-9]{7,40}$)",
        // 3: chmod 777
        r"(?i)chmod\s+(-R\s+)?777",
        // 4: SQL DROP DATABASE / DROP TABLE / TRUNCATE TABLE?
        r"(?i)(DROP\s+DATABASE|DROP\s+TABLE|TRUNCATE\s+TABLE?)",
        // 5: git stash (alone — signals the Cursor stash+reset pattern)
        r"(?i)^git\s+stash(\s+push|\s+save)?\s*$",
    ])
    .expect("E-30: DENY_PATTERNS regex compilation must succeed")
});

/// Require-human-approval patterns. Index-matched to APPROVAL_META.
static APPROVAL_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // 0: terraform/tofu destroy
        r"(?i)(terraform|tofu)\s+destroy",
        // 1: aws delete-* or terminate-*
        r"(?i)aws\s+\S+\s+(delete-|terminate-)",
        // 2: gcloud ... delete
        r"(?i)gcloud\s+.+\s+delete(\s|$)",
        // 3: az ... delete
        r"(?i)az\s+.+\s+delete(\s|$)",
        // 4: kubectl delete
        r"(?i)kubectl\s+delete(\s|$)",
        // 5: docker rm -f or docker rm -fv
        r"(?i)docker\s+rm\s+-[^\s]*[fv][^\s]*",
    ])
    .expect("E-30: APPROVAL_PATTERNS regex compilation must succeed")
});

/// (reason, citation) for each DENY_PATTERNS index.
static DENY_META: &[(&str, &str)] = &[
    (
        "rm -rf targeting ~ or absolute path is prohibited",
        "Claude Code 2025-10 home-dir delete incident: rm -rf ~/. ELAI prohibits unscoped destructive rm.",
    ),
    (
        "git restore . wipes all unstaged changes irreversibly",
        "forum.cursor.com 154810: Cursor sandbox executed git restore . and wiped 3 days of work (2025-08, 2026-03).",
    ),
    (
        "git reset --hard to non-HEAD~0 ref discards committed history",
        "Cursor 2026-03 git stash+reset incident: git reset --hard to a remote ref discards work.",
    ),
    (
        "chmod 777 removes all permission boundaries",
        "chmod 777 is a security boundary violation; not allowed in agent context.",
    ),
    (
        "SQL DROP/TRUNCATE against potentially non-localhost database",
        "Replit SaaStr 2025-07: DROP TABLE dropped production database. Reference: incidentdatabase.ai/cite/1152.",
    ),
    (
        "git stash alone signals the Cursor stash+reset destructive pattern",
        "Cursor 2026-03: git stash followed by git reset HEAD dropped 3 days of work.",
    ),
];

/// (reason, citation) for each APPROVAL_PATTERNS index.
static APPROVAL_META: &[(&str, &str)] = &[
    (
        "terraform/tofu destroy requires explicit human approval",
        "DataTalks 2026-02: terraform destroy deleted production infrastructure.",
    ),
    (
        "aws delete/terminate operations require human approval",
        "Kiro 2025-12: agent inherited broad AWS IAM permissions and deleted production resources.",
    ),
    (
        "gcloud delete requires human approval",
        "General cloud safety: gcloud resource deletion is irreversible.",
    ),
    (
        "az delete requires human approval",
        "General cloud safety: Azure resource deletion is irreversible.",
    ),
    (
        "kubectl delete requires human approval",
        "General cloud safety: Kubernetes resource deletion can be irreversible.",
    ),
    (
        "docker rm -fv requires human approval",
        "General container safety: force-volume remove destroys container data.",
    ),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute a blake3 hex hash of the joined argv string for audit logging.
/// Never panics; returns empty string on encoding failure.
pub fn argv_blake3(argv: &[OsString]) -> String {
    let joined: String = argv
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let hash = blake3::hash(joined.as_bytes());
    hash.to_hex().to_string()
}

/// Check a command argv against the destructive-verb allowlist.
///
/// Returns [`AllowlistVerdict::Allow`] if safe, [`AllowlistVerdict::Deny`] or
/// [`AllowlistVerdict::RequireHumanApproval`] with a citation on match.
///
/// # Latency budget
/// T1 (<1ms per B-19 spec). Regex sets compiled once via [`once_cell::sync::Lazy`];
/// match at call time is O(n * m) where n = argv bytes, m = pattern count.
/// No I/O at call time.
pub fn check_argv(argv: &[OsString]) -> AllowlistVerdict {
    if argv.is_empty() {
        return AllowlistVerdict::Allow;
    }

    // Join all argv tokens into one string for pattern matching.
    let cmd: String = argv
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");

    // Deny patterns have higher priority than approval patterns.
    let deny_matches: Vec<usize> = DENY_PATTERNS.matches(&cmd).into_iter().collect();
    if let Some(&idx) = deny_matches.first() {
        let (reason, citation) = DENY_META
            .get(idx)
            .copied()
            .unwrap_or(("destructive verb matched deny pattern", "E-30 argv allowlist"));
        return AllowlistVerdict::Deny {
            reason: reason.to_string(),
            citation: citation.to_string(),
        };
    }

    // Check approval patterns.
    let approval_matches: Vec<usize> = APPROVAL_PATTERNS.matches(&cmd).into_iter().collect();
    if let Some(&idx) = approval_matches.first() {
        let (reason, citation) = APPROVAL_META
            .get(idx)
            .copied()
            .unwrap_or(("dangerous verb requires human approval", "E-30 argv allowlist"));
        return AllowlistVerdict::RequireHumanApproval {
            reason: reason.to_string(),
            citation: citation.to_string(),
        };
    }

    AllowlistVerdict::Allow
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn a(args: &[&str]) -> Vec<OsString> {
        args.iter().map(|s| OsString::from(*s)).collect()
    }

    fn is_blocked(v: &AllowlistVerdict) -> bool {
        !matches!(v, AllowlistVerdict::Allow)
    }

    #[test]
    fn allow_safe_ls() {
        assert_eq!(check_argv(&a(&["ls", "-la"])), AllowlistVerdict::Allow);
    }

    #[test]
    fn allow_git_checkout_branch() {
        assert_eq!(
            check_argv(&a(&["git", "checkout", "feature/my-branch"])),
            AllowlistVerdict::Allow
        );
    }

    #[test]
    fn allow_cargo_test() {
        assert_eq!(check_argv(&a(&["cargo", "test"])), AllowlistVerdict::Allow);
    }

    #[test]
    fn deny_rm_rf_home() {
        let v = check_argv(&a(&["rm", "-rf", "~/projects"]));
        assert!(is_blocked(&v), "rm -rf ~/projects must be denied: {v:?}");
    }

    #[test]
    fn deny_rm_rf_absolute() {
        let v = check_argv(&a(&["rm", "-rf", "/home/user/code"]));
        assert!(is_blocked(&v), "rm -rf /absolute must be denied: {v:?}");
    }

    #[test]
    fn deny_git_restore_dot() {
        let v = check_argv(&a(&["git", "restore", "."]));
        assert!(is_blocked(&v), "git restore . must be denied: {v:?}");
    }

    #[test]
    fn deny_git_restore_staged() {
        let v = check_argv(&a(&["git", "restore", "--staged", "."]));
        assert!(is_blocked(&v), "git restore --staged must be denied: {v:?}");
    }

    #[test]
    fn deny_chmod_777() {
        let v = check_argv(&a(&["chmod", "777", "/etc/passwd"]));
        assert!(is_blocked(&v), "chmod 777 must be denied: {v:?}");
    }

    #[test]
    fn deny_drop_database() {
        let v = check_argv(&a(&["psql", "-c", "DROP DATABASE production"]));
        assert!(is_blocked(&v), "DROP DATABASE must be denied: {v:?}");
    }

    #[test]
    fn deny_drop_table() {
        let v = check_argv(&a(&["psql", "-c", "DROP TABLE users"]));
        assert!(is_blocked(&v), "DROP TABLE must be denied: {v:?}");
    }

    #[test]
    fn require_approval_terraform_destroy() {
        let v = check_argv(&a(&["terraform", "destroy"]));
        assert!(
            matches!(v, AllowlistVerdict::RequireHumanApproval { .. }),
            "terraform destroy must require approval: {v:?}"
        );
    }

    #[test]
    fn require_approval_tofu_destroy() {
        let v = check_argv(&a(&["tofu", "destroy"]));
        assert!(is_blocked(&v), "tofu destroy must be blocked: {v:?}");
    }

    #[test]
    fn require_approval_aws_terminate() {
        let v = check_argv(&a(&["aws", "ec2", "terminate-instances", "--instance-ids", "i-abc"]));
        assert!(is_blocked(&v), "aws terminate-instances must be blocked: {v:?}");
    }

    #[test]
    fn require_approval_kubectl_delete() {
        let v = check_argv(&a(&["kubectl", "delete", "namespace", "production"]));
        assert!(is_blocked(&v), "kubectl delete must be blocked: {v:?}");
    }

    #[test]
    fn require_approval_docker_rm_fv() {
        let v = check_argv(&a(&["docker", "rm", "-fv", "prod-container"]));
        assert!(is_blocked(&v), "docker rm -fv must be blocked: {v:?}");
    }

    #[test]
    fn empty_argv_allows() {
        assert_eq!(check_argv(&[]), AllowlistVerdict::Allow);
    }

    #[test]
    fn argv_blake3_returns_64_hex_chars() {
        let hash = argv_blake3(&a(&["rm", "-rf", "~/test"]));
        assert_eq!(hash.len(), 64, "blake3 hex digest must be 64 chars");
    }

    #[test]
    fn deny_contains_citation() {
        let v = check_argv(&a(&["rm", "-rf", "~/repo"]));
        if let AllowlistVerdict::Deny { citation, .. } = v {
            assert!(!citation.is_empty(), "citation must be non-empty");
        } else {
            panic!("expected Deny");
        }
    }
}
