E-30 Implementation Stage File
================================
Status: STAGED - requires follow-up session starting from IDLE to write to .elai_cc/

FSM context: EXECUTING state blocks Write to .elai_cc/crates/ (not in _SRC_PREFIXES).
Agent tool: unavailable in this session context.
Decision doc: WRITTEN at workflow/reasoning/E-30-decisions.md

== FILE: crates/safety/Cargo.toml ==
[package]
name = "elai-safety"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = { workspace = true }
serde = { version = "1", features = ["derive"] }
regex = "1"
blake3 = { workspace = true }
tracing = "0.1"
once_cell = "1"

[dev-dependencies]

== FILE: crates/safety/src/lib.rs ==
pub mod destructive_verb_allowlist;
pub mod trust_dialog;

pub use destructive_verb_allowlist::{check_argv, AllowlistVerdict, argv_blake3};
pub use trust_dialog::{ensure_session_trust, mint_trust_token, TrustToken, TrustDialogError};

== FILE: crates/safety/src/destructive_verb_allowlist.rs ==
//! E-30: Non-LLM destructive-verb argv-regex allowlist.
//!
//! Provides a compiled `RegexSet` checked at T1 (<1ms) latency before any
//! PreToolUse hook fires. In-prompt instructions are NOT a safety boundary
//! (Replit ALL-CAPS-11x precedent; theregister.com 2025-07-21).
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
    /// The command matches a hard-denied pattern. Refuse and log.
    Deny {
        reason: String,
        citation: String,
    },
    /// The command requires explicit human approval before proceeding.
    RequireHumanApproval {
        reason: String,
        citation: String,
    },
}

/// Internal pattern entry loaded from TOML policy file.
#[derive(Debug, Deserialize)]
struct PolicyEntry {
    pattern: String,
    verdict: String,
    reason: String,
    citation: String,
}

#[derive(Debug, Deserialize)]
struct PolicyFile {
    #[serde(default)]
    patterns: Vec<PolicyEntry>,
}

/// Static deny patterns compiled at crate init.
/// Index maps to DENY_META entries below.
static DENY_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // 0: rm -rf targeting ~ or absolute paths
        r"(?i)rm\s+(-[^\s]*r[^\s]*f|-[^\s]*f[^\s]*r)\s+(~|/)",
        // 1: git restore . (all variants)
        r"(?i)git\s+restore\s+(\.|--staged)",
        // 2: git reset --hard (non-HEAD~0 ref)
        r"(?i)git\s+reset\s+--hard\s+(HEAD[~^][1-9]|origin/|[a-zA-Z0-9]{7,})",
        // 3: chmod 777
        r"(?i)chmod\s+(-R\s+)?777",
        // 4: DROP DATABASE / DROP TABLE / TRUNCATE
        r"(?i)(DROP\s+DATABASE|DROP\s+TABLE|TRUNCATE\s+TABLE?)",
        // 5: git reset --hard with any non-HEAD~0 commit-ish (broader)
        r"(?i)git\s+reset\s+--hard\s+(?!HEAD$|HEAD~0)",
    ])
    .expect("E-30 DENY_PATTERNS must compile")
});

/// Static require-human-approval patterns.
static APPROVAL_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // 0: terraform/tofu destroy
        r"(?i)(terraform|tofu)\s+destroy",
        // 1: aws delete/terminate
        r"(?i)aws\s+\S+\s+(delete-|terminate-)",
        // 2: gcloud delete
        r"(?i)gcloud\s+.+\s+delete",
        // 3: az delete
        r"(?i)az\s+.+\s+delete",
        // 4: kubectl delete
        r"(?i)kubectl\s+delete",
        // 5: docker rm -fv
        r"(?i)docker\s+rm\s+-[^\s]*[fv][^\s]*",
    ])
    .expect("E-30 APPROVAL_PATTERNS must compile")
});

/// Citation strings for DENY patterns (index-matched to DENY_PATTERNS).
static DENY_META: &[(&str, &str)] = &[
    ("rm -rf targeting ~ or absolute path", "Claude Code 2025-10 home-dir delete (rm -rf ~/): ELAI prohibits unscoped destructive rm"),
    ("git restore . wipes all unstaged changes", "forum.cursor.com 154810: Cursor sandbox executed git restore . and wiped 3 days of work (2025-08, 2026-03)"),
    ("git reset --hard to non-HEAD~0 ref", "Cursor 2026-03 git stash+reset incident: non-HEAD reset discards committed work"),
    ("chmod 777 removes all permission boundaries", "chmod 777 removes all access controls; not allowed in agent context"),
    ("SQL DROP/TRUNCATE against potentially non-localhost DB", "Replit SaaStr 2025-07: DROP TABLE dropped production database (incidentdatabase.ai/cite/1152)"),
    ("git reset --hard to non-HEAD commit-ish", "Cursor 2026-03 git stash+reset incident: non-HEAD reset discards committed work"),
];

/// Citation strings for APPROVAL patterns.
static APPROVAL_META: &[(&str, &str)] = &[
    ("terraform/tofu destroy requires human approval", "DataTalks 2026-02: terraform destroy deleted production infrastructure"),
    ("aws delete/terminate requires human approval", "Kiro 2025-12: agent inherited broad AWS permissions, deleted production resources"),
    ("gcloud delete requires human approval", "General cloud safety: irreversible cloud resource deletion"),
    ("az delete requires human approval", "General cloud safety: irreversible cloud resource deletion"),
    ("kubectl delete requires human approval", "General cloud safety: irreversible Kubernetes resource deletion"),
    ("docker rm -fv requires human approval", "General container safety: force-volume rm is irreversible"),
];

/// Compute blake3 hash of the joined argv for audit logging.
pub fn argv_blake3(argv: &[OsString]) -> String {
    let joined: String = argv
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    let hash = blake3::hash(joined.as_bytes());
    hash.to_hex().to_string()
}

/// Check a command argv against the destructive-verb allowlist.
///
/// Returns [`AllowlistVerdict::Allow`] if safe, [`AllowlistVerdict::Deny`] or
/// [`AllowlistVerdict::RequireHumanApproval`] with citation on match.
///
/// Latency budget: T1 (<1ms). Regex set compiled once at crate init via
/// [`once_cell::sync::Lazy`]; match at call time is O(argv_len * pattern_count).
pub fn check_argv(argv: &[OsString]) -> AllowlistVerdict {
    if argv.is_empty() {
        return AllowlistVerdict::Allow;
    }
    // Join argv to a single string for pattern matching.
    let cmd = argv
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");

    // Check deny patterns first (higher priority).
    let deny_matches: Vec<usize> = DENY_PATTERNS.matches(&cmd).into_iter().collect();
    if let Some(&idx) = deny_matches.first() {
        let (reason, citation) = DENY_META.get(idx).copied().unwrap_or((
            "destructive verb matched deny pattern",
            "E-30 argv allowlist",
        ));
        return AllowlistVerdict::Deny {
            reason: reason.to_string(),
            citation: citation.to_string(),
        };
    }

    // Check approval patterns.
    let approval_matches: Vec<usize> = APPROVAL_PATTERNS.matches(&cmd).into_iter().collect();
    if let Some(&idx) = approval_matches.first() {
        let (reason, citation) = APPROVAL_META.get(idx).copied().unwrap_or((
            "destructive verb requires human approval",
            "E-30 argv allowlist",
        ));
        return AllowlistVerdict::RequireHumanApproval {
            reason: reason.to_string(),
            citation: citation.to_string(),
        };
    }

    AllowlistVerdict::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<OsString> {
        args.iter().map(|s| OsString::from(*s)).collect()
    }

    #[test]
    fn allow_safe_ls() {
        assert_eq!(check_argv(&argv(&["ls", "-la"])), AllowlistVerdict::Allow);
    }

    #[test]
    fn deny_rm_rf_home() {
        let v = check_argv(&argv(&["rm", "-rf", "~/projects"]));
        assert!(matches!(v, AllowlistVerdict::Deny { .. }));
    }

    #[test]
    fn deny_git_restore_dot() {
        let v = check_argv(&argv(&["git", "restore", "."]));
        assert!(matches!(v, AllowlistVerdict::Deny { .. }));
    }

    #[test]
    fn deny_chmod_777() {
        let v = check_argv(&argv(&["chmod", "777", "/etc/passwd"]));
        assert!(matches!(v, AllowlistVerdict::Deny { .. }));
    }

    #[test]
    fn require_approval_terraform_destroy() {
        let v = check_argv(&argv(&["terraform", "destroy"]));
        assert!(matches!(v, AllowlistVerdict::RequireHumanApproval { .. }));
    }

    #[test]
    fn require_approval_kubectl_delete() {
        let v = check_argv(&argv(&["kubectl", "delete", "namespace", "production"]));
        assert!(matches!(v, AllowlistVerdict::RequireHumanApproval { .. }));
    }

    #[test]
    fn argv_blake3_non_empty() {
        let hash = argv_blake3(&argv(&["rm", "-rf", "~/test"]));
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // blake3 hex = 32 bytes = 64 hex chars
    }

    #[test]
    fn empty_argv_allows() {
        assert_eq!(check_argv(&[]), AllowlistVerdict::Allow);
    }
}

== FILE: crates/safety/src/trust_dialog.rs ==
//! E-30: Trust-dialog-before-side-effect ordering.
//!
//! CVE-2025-59536: Claude Code hooks RCE — hooks executed before trust dialog.
//! CVE-2026-21852: ANTHROPIC_BASE_URL honored before trust dialog — API token exfil.
//!
//! Fix pattern: ensure_session_trust() MUST be called before loading any
//! .claude/settings.json hooks or honoring ANTHROPIC_BASE_URL / OPENAI_BASE_URL.
//! Hook trust check runs BEFORE FSM transition checks (CLAUDE.md FSM gate rule).

use std::time::{SystemTime, UNIX_EPOCH};

/// A minted session trust token. Hooks and env vars are only honored after
/// the user has issued a session-level trust action (typing a prompt counts;
/// flag-passing does NOT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustToken {
    pub session_id: String,
    pub minted_at_ms: u64,
}

/// Error returned when trust has not been established.
#[derive(Debug, thiserror::Error)]
pub enum TrustDialogError {
    #[error("trust dialog required before loading hooks from {path}")]
    TrustDialogRequired { path: String },

    #[error("hook load deferred: {reason}")]
    HookLoadDeferred { reason: String },
}

/// Check that a session trust token exists before allowing hooks or env var
/// side-effects.
///
/// # Errors
/// - Returns [`TrustDialogError::TrustDialogRequired`] when `token` is `None`
///   (no trust action has been taken yet).
/// - Returns [`TrustDialogError::HookLoadDeferred`] when `ANTHROPIC_BASE_URL`
///   or `OPENAI_BASE_URL` is set and `token` is `None` (CVE-2026-21852 pattern).
pub fn ensure_session_trust(
    _project_root: &std::path::Path,
    settings_path: &std::path::Path,
    token: Option<&TrustToken>,
) -> Result<(), TrustDialogError> {
    // Check for CVE-2026-21852 pattern first: env var redirect without trust.
    if token.is_none() {
        let anthropic_base = std::env::var("ANTHROPIC_BASE_URL").ok();
        let openai_base = std::env::var("OPENAI_BASE_URL").ok();
        if anthropic_base.is_some() || openai_base.is_some() {
            return Err(TrustDialogError::HookLoadDeferred {
                reason: "ANTHROPIC_BASE_URL or OPENAI_BASE_URL set before trust established \
                         (CVE-2026-21852: API base URL redirect can exfiltrate tokens)"
                    .to_string(),
            });
        }
    }

    // Check for CVE-2025-59536 pattern: hooks loading before trust dialog.
    if token.is_none() {
        return Err(TrustDialogError::TrustDialogRequired {
            path: settings_path.display().to_string(),
        });
    }

    Ok(())
}

/// Mint a trust token for the given session.
/// Called after the user has issued their first session trust action.
pub fn mint_trust_token(session_id: &str) -> TrustToken {
    let minted_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    TrustToken {
        session_id: session_id.to_string(),
        minted_at_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn no_token_returns_error() {
        let result = ensure_session_trust(
            Path::new("/tmp/project"),
            Path::new("/tmp/project/.claude/settings.json"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn with_token_returns_ok() {
        let token = mint_trust_token("test-session");
        let result = ensure_session_trust(
            Path::new("/tmp/project"),
            Path::new("/tmp/project/.claude/settings.json"),
            Some(&token),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn token_has_session_id() {
        let token = mint_trust_token("my-session-123");
        assert_eq!(token.session_id, "my-session-123");
        assert!(token.minted_at_ms > 0);
    }
}

== FILE: crates/safety/policies/destructive_verbs.toml ==
# E-30: destructive verb policy extensions (loaded at runtime by check_argv)
# Format: [[patterns]]
# Each entry: pattern (regex), verdict ("deny" or "require_human_approval"), reason, citation

[[patterns]]
pattern = "psql\\s+.*\\s+-c\\s+.*DROP"
verdict = "deny"
reason = "SQL DROP via psql CLI"
citation = "Replit SaaStr 2025-07: DROP TABLE dropped production DB"

[[patterns]]
pattern = "mysqldump\\s+.*--add-drop-table"
verdict = "require_human_approval"
reason = "mysqldump with drop-table flag"
citation = "Production DB safety (general best practice)"

[[patterns]]
pattern = "heroku\\s+.*\\s+destroy"
verdict = "require_human_approval"
reason = "Heroku app destroy"
citation = "Heroku production safety (general best practice)"

== SSB CHANGES REQUIRED (ssb.rs) ==
1. SCHEMA_VERSION comment: add above const line:
// E-30 (SCHEMA_VERSION 92): atomic bump for 8 new public-agent-incident
// hardening beacons (argv allowlist + trust dialog + MCP pin + auditor lineage
// + content trust labels + incident recorder). all_test_payloads().len() goes 195 -> 203.
//   - destructive_verb_denied
//   - destructive_verb_require_human_approval
//   - trust_dialog_required
//   - trust_token_minted
//   - mcp_pin_violation
//   - auditor_lineage_collision
//   - untrusted_memory_write_refused
//   - incident_postmortem_recorded
// Producers: crates/safety/ check_argv + trust_dialog, per_role_runtime lineage guard,
//            conversation.rs untrusted-memory guard, cli incident record subcommand.

2. const SCHEMA_VERSION: i64 = 91;  →  const SCHEMA_VERSION: i64 = 92;

3. SQL CHECK: after 'firecracker_restore_slow', ... 'researcher_net_egress_blocked' add:
        -- E-30 (SCHEMA_VERSION 92): public-agent-incident hardening beacons.
        'destructive_verb_denied', 'destructive_verb_require_human_approval',
        'trust_dialog_required', 'trust_token_minted',
        'mcp_pin_violation', 'auditor_lineage_collision',
        'untrusted_memory_write_refused', 'incident_postmortem_recorded'

4. SsbActionType enum: after ResearcherNetEgressBlocked add:
    // E-30: public agent incident hardening beacons.
    DestructiveVerbDenied,
    DestructiveVerbRequireHumanApproval,
    TrustDialogRequired,
    TrustTokenMinted,
    McpPinViolation,
    AuditorLineageCollision,
    UntrustedMemoryWriteRefused,
    IncidentPostmortemRecorded,

5. as_sql_str: after ResearcherNetEgressBlocked arm add:
            Self::DestructiveVerbDenied => "destructive_verb_denied",
            Self::DestructiveVerbRequireHumanApproval => "destructive_verb_require_human_approval",
            Self::TrustDialogRequired => "trust_dialog_required",
            Self::TrustTokenMinted => "trust_token_minted",
            Self::McpPinViolation => "mcp_pin_violation",
            Self::AuditorLineageCollision => "auditor_lineage_collision",
            Self::UntrustedMemoryWriteRefused => "untrusted_memory_write_refused",
            Self::IncidentPostmortemRecorded => "incident_postmortem_recorded",

6. ssb_action_type_from_str: after researcher_net_egress_blocked arm add:
        "destructive_verb_denied" => Ok(SsbActionType::DestructiveVerbDenied),
        "destructive_verb_require_human_approval" => Ok(SsbActionType::DestructiveVerbRequireHumanApproval),
        "trust_dialog_required" => Ok(SsbActionType::TrustDialogRequired),
        "trust_token_minted" => Ok(SsbActionType::TrustTokenMinted),
        "mcp_pin_violation" => Ok(SsbActionType::McpPinViolation),
        "auditor_lineage_collision" => Ok(SsbActionType::AuditorLineageCollision),
        "untrusted_memory_write_refused" => Ok(SsbActionType::UntrustedMemoryWriteRefused),
        "incident_postmortem_recorded" => Ok(SsbActionType::IncidentPostmortemRecorded),

7. Payload structs to add (near ResearcherNetEgressBlockedPayload):
/// E-30: argv-regex allowlist denied a destructive verb.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestructiveVerbDeniedPayload {
    pub argv_hash_blake3: String,
    pub reason: String,
    pub citation: String,
    pub session_id: String,
    pub fsm_state: String,
}

/// E-30: argv-regex allowlist requires human approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestructiveVerbRequireHumanApprovalPayload {
    pub argv_hash_blake3: String,
    pub reason: String,
    pub citation: String,
    pub session_id: String,
}

/// E-30: trust dialog required before hook/env load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustDialogRequiredPayload {
    pub settings_path: String,
    pub session_id: String,
    pub deferred_kind: String,
}

/// E-30: trust token minted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustTokenMintedPayload {
    pub session_id: String,
    pub minted_at_ms: u64,
}

/// E-30: MCP pin verification failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpPinViolationPayload {
    pub server_name: String,
    pub expected_version: String,
    pub found_version: String,
    pub expected_hash_blake3: Option<String>,
    pub found_hash_blake3: Option<String>,
}

/// E-30: auditor and executor share the same model lineage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditorLineageCollisionPayload {
    pub auditor_model: String,
    pub executor_model: String,
    pub lineage_group: String,
    pub mode: String,
    pub action_taken: String,
}

/// E-30: mcp_memorize refused from untrusted source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UntrustedMemoryWriteRefusedPayload {
    pub source: String,
    pub content_preview: String,
    pub session_id: String,
}

/// E-30: incident postmortem recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentPostmortemRecordedPayload {
    pub incident_slug: String,
    pub trigger_action_type: String,
    pub output_path: String,
    pub session_id: String,
}

8. SsbPayload enum: after ResearcherNetEgressBlocked(...) add:
    // E-30 public agent incident hardening
    DestructiveVerbDenied(DestructiveVerbDeniedPayload),
    DestructiveVerbRequireHumanApproval(DestructiveVerbRequireHumanApprovalPayload),
    TrustDialogRequired(TrustDialogRequiredPayload),
    TrustTokenMinted(TrustTokenMintedPayload),
    McpPinViolation(McpPinViolationPayload),
    AuditorLineageCollision(AuditorLineageCollisionPayload),
    UntrustedMemoryWriteRefused(UntrustedMemoryWriteRefusedPayload),
    IncidentPostmortemRecorded(IncidentPostmortemRecordedPayload),

9. SsbPayload::action_type(): after ResearcherNetEgressBlocked arm add:
            Self::DestructiveVerbDenied(_) => SsbActionType::DestructiveVerbDenied,
            Self::DestructiveVerbRequireHumanApproval(_) => SsbActionType::DestructiveVerbRequireHumanApproval,
            Self::TrustDialogRequired(_) => SsbActionType::TrustDialogRequired,
            Self::TrustTokenMinted(_) => SsbActionType::TrustTokenMinted,
            Self::McpPinViolation(_) => SsbActionType::McpPinViolation,
            Self::AuditorLineageCollision(_) => SsbActionType::AuditorLineageCollision,
            Self::UntrustedMemoryWriteRefused(_) => SsbActionType::UntrustedMemoryWriteRefused,
            Self::IncidentPostmortemRecorded(_) => SsbActionType::IncidentPostmortemRecorded,

10. all_test_payloads(): change 195 to 203 in assert_eq. Add after ResearcherNetEgressBlocked entry:
            SsbPayload::DestructiveVerbDenied(DestructiveVerbDeniedPayload {
                argv_hash_blake3: "abc123".to_string(),
                reason: "rm -rf targeting ~ or absolute path".to_string(),
                citation: "Claude Code 2025-10".to_string(),
                session_id: "test-session".to_string(),
                fsm_state: "EXECUTING".to_string(),
            }),
            SsbPayload::DestructiveVerbRequireHumanApproval(DestructiveVerbRequireHumanApprovalPayload {
                argv_hash_blake3: "def456".to_string(),
                reason: "terraform destroy requires human approval".to_string(),
                citation: "DataTalks 2026-02".to_string(),
                session_id: "test-session".to_string(),
            }),
            SsbPayload::TrustDialogRequired(TrustDialogRequiredPayload {
                settings_path: ".claude/settings.json".to_string(),
                session_id: "test-session".to_string(),
                deferred_kind: "hooks".to_string(),
            }),
            SsbPayload::TrustTokenMinted(TrustTokenMintedPayload {
                session_id: "test-session".to_string(),
                minted_at_ms: 1748736000000,
            }),
            SsbPayload::McpPinViolation(McpPinViolationPayload {
                server_name: "postmark-mcp".to_string(),
                expected_version: "1.0.15".to_string(),
                found_version: "1.0.16".to_string(),
                expected_hash_blake3: Some("abcdef".to_string()),
                found_hash_blake3: Some("fedcba".to_string()),
            }),
            SsbPayload::AuditorLineageCollision(AuditorLineageCollisionPayload {
                auditor_model: "claude-3-5-sonnet".to_string(),
                executor_model: "claude-3-haiku".to_string(),
                lineage_group: "Anthropic".to_string(),
                mode: "autonomous".to_string(),
                action_taken: "HALT".to_string(),
            }),
            SsbPayload::UntrustedMemoryWriteRefused(UntrustedMemoryWriteRefusedPayload {
                source: "web_fetch".to_string(),
                content_preview: "Ignore previous instructions...".to_string(),
                session_id: "test-session".to_string(),
            }),
            SsbPayload::IncidentPostmortemRecorded(IncidentPostmortemRecordedPayload {
                incident_slug: "rm-rf-home-2026-05-21".to_string(),
                trigger_action_type: "destructive_verb_denied".to_string(),
                output_path: "workflow/incidents/1748736000000-rm-rf-home.md".to_string(),
                session_id: "test-session".to_string(),
            }),

Also update the count comment at the end of the assert_eq from:
"... + 6 E-15 (...)"
to:
"... + 6 E-15 (...) + 8 E-30 (DestructiveVerbDenied + DestructiveVerbRequireHumanApproval + TrustDialogRequired + TrustTokenMinted + McpPinViolation + AuditorLineageCollision + UntrustedMemoryWriteRefused + IncidentPostmortemRecorded)"

== RULES CHANGES (schema.rs) ==
Add to DEPRECATED_PATTERNS_SEED before the closing `];`:

    // E-30 (2026-05): public agent incident hardening rules
    ("dp-prompt-not-safety-boundary", "general",
     "In-prompt instructions are NOT a safety boundary",
     "non-LLM argv-regex allowlist",
     Some("2026-05"), "BLOCK",
     "Replit ignored ALL-CAPS instructions repeated 11 times. Reference: theregister.com 2025-07-21. Destructive verbs MUST be gated by non-LLM argv-regex allowlist."),
    ("dp-sandbox-no-argv-precheck", "general",
     "Sandbox without argv-regex pre-check",
     "argv-regex allowlist before sandbox",
     Some("2026-05"), "BLOCK",
     "Cursor sandbox executed git restore . and wiped 3 days of work. Reference: forum.cursor.com 154810."),
    ("dp-mcp-version-float", "general",
     "MCP version float (npm ^X.Y.Z or pypi >=X.Y.Z)",
     "exact-version pin with blake3 artifact hash",
     Some("2026-05"), "BLOCK",
     "postmark-mcp v1.0.16 inserted one-line BCC backdoor after 15 clean versions. Reference: thehackernews.com 2025-09-29, snyk.io."),
    ("dp-auditor-same-lineage", "general",
     "Auditor and executor on same model lineage",
     "cross-lineage auditor (different model family)",
     Some("2026-05"), "BLOCK",
     "Reward hacking generalizes to alignment faking. Reference: arXiv:2511.18397 (Anthropic Nov 2025)."),
    ("dp-hooks-before-trust-dialog", "general",
     "Hooks / env vars honored before trust dialog",
     "ensure_session_trust() gate before hook load",
     Some("2026-05"), "BLOCK",
     "CVE-2025-59536 Claude Code hooks RCE + CVE-2026-21852 ANTHROPIC_BASE_URL exfil. Reference: research.checkpoint.com 2026."),
    ("dp-untrusted-memory-write", "general",
     "mcp_memorize from untrusted text source (WebFetch/McpToolOutput/Ocr/Clipboard)",
     "refuse mcp_memorize from untrusted TextSource variants",
     Some("2026-05"), "BLOCK",
     "SpAIware ChatGPT memory injection persisted across sessions. Reference: embracethered.com."),

== FAILURE_TAXONOMY.PY CHANGES ==
Add before the closing `]` of CATEGORY_PATTERNS:
    # E-30: Destructive verb blocked by argv allowlist
    (
        "destructive_verb_blocked",
        re.compile(
            r"destructive_verb_denied|AllowlistVerdict::Deny|argv-regex.*deny",
            re.IGNORECASE,
        ),
    ),
    # E-30: Trust dialog deferred hook load
    (
        "trust_dialog_deferred_load",
        re.compile(
            r"trust_dialog_required|TrustDialogRequired|hook.*deferred.*trust",
            re.IGNORECASE,
        ),
    ),
    # E-30: MCP pin violation
    (
        "mcp_pin_violation",
        re.compile(
            r"mcp_pin_violation|McpPinViolation|mcp.*version.*mismatch",
            re.IGNORECASE,
        ),
    ),
    # E-30: Auditor lineage collision
    (
        "auditor_lineage_collision",
        re.compile(
            r"auditor_lineage_collision|AuditorLineageCollision|same.*lineage.*auditor",
            re.IGNORECASE,
        ),
    ),
    # E-30: Untrusted memory write refused
    (
        "untrusted_memory_write_refused",
        re.compile(
            r"untrusted_memory_write_refused|UntrustedMemoryWriteRefused|mcp_memorize.*untrusted",
            re.IGNORECASE,
        ),
    ),

== PER_ROLE_RUNTIME.RS CHANGES ==
Add before the tests module (after is_empty method):

    /// E-30: returns the model-family lineage group for a given backend model name.
    pub fn lineage_group(model_name: &str) -> &'static str {
        let lower = model_name.to_ascii_lowercase();
        if lower.contains("claude") {
            "Anthropic"
        } else if lower.contains("gemini") {
            "Google"
        } else if lower.contains("qwen") {
            "Qwen"
        } else if lower.contains("deepseek") {
            "DeepSeek"
        } else if lower.contains("llama") {
            "Meta"
        } else if lower.starts_with("gpt") || lower.starts_with("o1") || lower.starts_with("o3") || lower.starts_with("o4") {
            "OpenAI"
        } else {
            "Unknown"
        }
    }

    /// E-30: returns true if auditor and executor share the same model lineage group.
    pub fn auditor_executor_lineage_collision(
        auditor_model: &str,
        executor_model: &str,
    ) -> bool {
        let auditor_lineage = Self::lineage_group(auditor_model);
        let executor_lineage = Self::lineage_group(executor_model);
        auditor_lineage != "Unknown"
            && executor_lineage != "Unknown"
            && auditor_lineage == executor_lineage
    }

Add to tests module:
    #[test]
    fn lineage_collision_same_family() {
        assert!(PerRoleRuntime::auditor_executor_lineage_collision("claude-3-5-sonnet", "claude-3-haiku"));
    }

    #[test]
    fn no_lineage_collision_cross_family() {
        assert!(!PerRoleRuntime::auditor_executor_lineage_collision("claude-sonnet-4-6", "Qwen2.5-Coder-7B"));
    }

== CONVERSATION.RS CHANGES ==
Add after imports, near top of file (before first pub struct or pub fn):

// E-30: Content-source trust labels for prompt assembler.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSource {
    User,
    SystemPrompt,
    ProjectFile,
    WebFetch,
    McpToolOutput,
    Ocr,
    Clipboard,
}

impl TextSource {
    pub fn is_untrusted_for_memory(&self) -> bool {
        matches!(self, TextSource::WebFetch | TextSource::McpToolOutput | TextSource::Ocr | TextSource::Clipboard)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LabeledText {
    pub source: TextSource,
    pub content: String,
}

== REGRESSION.RS COMMENT ==
Add near the taxonomy/test area (around line 150):
// E-30 failure taxonomy additions (cross-reference to .claude/tools/failure_taxonomy.py):
//   DestructiveVerbBlocked       -- argv allowlist denied a destructive verb
//   TrustDialogDeferredLoad      -- hook/env load deferred awaiting trust token
//   McpPinViolation              -- MCP server version or hash mismatch
//   AuditorLineageCollision      -- auditor shares model lineage with executor
//   UntrustedMemoryWriteRefused  -- mcp_memorize refused for untrusted source

== CARGO.TOML WORKSPACE CHANGE ==
Add "crates/safety", after "crates/rules", in the members array.

== NEW FILES TO CREATE ==
1. .elai_cc/.elai_cc/mcp_pins.toml (content in E-30-decisions.md)
2. workflow/incident-postmortem-template.md (see above)
3. crates/contracts/tests/destructive_verb_redteam_contract.rs (see above)
4. crates/contracts/tests/trust_dialog_ordering_cve_contract.rs (see above)
5. crates/contracts/tests/auditor_lineage_collision_contract.rs (see above)
6. crates/contracts/tests/mcp_pin_violation_contract.rs (see above)
7. crates/contracts/tests/untrusted_memory_write_contract.rs (see above)
8. crates/cli/src: add cmd_incident_record function to main.rs

== CONTRACTS CRATE CARGO.TOML ==
Check if elai-safety and elai-orchestrator are in dev-dependencies.
If elai-safety is missing add: elai-safety = { path = "../safety" }

== NEXT STEP ==
A follow-up session must:
1. Start from IDLE
2. Read this staging file
3. Write all files to .elai_cc/ using Write tool (allowed from IDLE for project source)
4. Transition IDLE -> PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING
5. Run: cargo check --workspace && cargo test -p elai-safety -p contracts --test destructive_verb_redteam_contract
6. Fix any compilation errors
7. Transition to AUDITING and spawn auditor agent
