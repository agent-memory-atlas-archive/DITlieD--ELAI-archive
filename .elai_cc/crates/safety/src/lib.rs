//! elai-safety: Non-LLM destructive-verb allowlist + trust-dialog ordering.
//!
//! E-30: Ships the top-priority public-agent-incident mitigations:
//! (1) argv-regex allowlist at T1 (<1ms) before PreToolUse hook fires.
//! (2) Trust-dialog-before-side-effect ordering (CVE-2025-59536 / CVE-2026-21852).
//!
//! E-37: layers prompt-injection-defense modules ABOVE the E-24 mcp_schema_pin
//! + sanitize_result + llm_guard surface and E-30 destructive_verb_allowlist
//! + trust_dialog surface. Six new modules implement the OWASP LLM Top 10
//! LLM01 Prompt Injection defenses (Spotlight envelope, per-tool description
//! pin, PromptArmor classifier, provenance threading, CaMeL-lite capability
//! gate, injection classifier role). All ship behind wire-dark feature gates
//! per D-12/D-14/D-15/D-16/D-18 commented-stanza precedent; activation lives
//! in elai_cc.toml `[safety.*]` stanzas + ELAI_*_ENABLE env vars.
//!
//! Modules are flat siblings under `src/` (matches the E-30 layout in
//! `destructive_verb_allowlist.rs` + `trust_dialog.rs`); no sub-directory
//! tree (single-purpose modules per concern).

pub mod destructive_verb_allowlist;
pub mod trust_dialog;

// E-37 layered modules (OWASP LLM Top 10 LLM01 defenses). Order matches the
// dispatch chain: provenance threads through everything, spotlight wraps
// untrusted ingest, prompt_armor + injection_classifier classify, capability_gate
// rejects argv with tainted provenance, tool_description_pin verifies per-tool
// description blake3 atop E-24 list-level pin.
pub mod provenance;
pub mod spotlight;
pub mod prompt_armor;
pub mod capability_gate;
pub mod tool_description_pin;
pub mod injection_classifier;
// G-1: data-plane taint propagation (content-keyed side-table + dependency-
// screened join over the provenance labels above). Lights up the wire-dark
// capability_gate at the destructive sink.
pub mod taint_flow;
// G-3: deterministic runtime blast-radius gates. Pure, fail-closed verdict
// functions (post-expansion FS containment + host-side egress IP-pin) the
// orchestrator wires at run_bash / translate_tool_request / fetch_and_extract.
// Cross-platform Tier-1 (std::fs + std::net); no orchestrator dependency.
pub mod blast_radius;

pub use destructive_verb_allowlist::{check_argv, AllowlistVerdict, argv_blake3};
pub use trust_dialog::{ensure_session_trust, mint_trust_token, TrustToken, TrustDialogError};

// E-37 re-exports for downstream consumers (orchestrator, plugins, cli, dashboard).
pub use provenance::{Provenance, Provenanced, TextSource, TrustLevel};
pub use spotlight::{wrap_spotlight, extract_spotlight, SpotlightedSpan, SpotlightError};
pub use prompt_armor::{classify_injection, classify_with_regex, ClassifierVerdict, InjectionClassifier, HeuristicClassifier, PromptArmorError};
pub use capability_gate::{allow_tool_with_origin, CapabilityVerdict, CapabilityGateError, is_destructive_tool};
pub use taint_flow::{join_screened, seed_from_verdict, tainted_context, TaintedContext};
pub use blast_radius::{
    check_egress_host, check_fs_target, expand_fs_token, resolve_host_std, worktree_root_blake3,
    BlastRadiusKind, BlastRadiusVerdict, ExpandedToken,
};
pub use tool_description_pin::{pin_or_verify_tool, pin_tool_description, update_pin_tool, verify_pin, DescPinStatus, DescPinRecord, Tool, ToolDescriptionPinError};
pub use injection_classifier::{InjectionClassifierRole, INJECTION_CLASSIFIER_ROLE_LABEL};
