//! E-11 catalog+recommender slice -- prior-art registry citing the
//! 2024-2026 idea-to-app survey + grounded report findings.
//!
//! Source-of-truth: `workflow/improvement-sweep-2026-05-19/reports/
//! 11-idea-to-saas-prior-art.md`. Each entry carries:
//!
//!   * `name`            -- product / report name as it appears in the survey
//!   * `url`             -- public landing or canonical citation URL
//!   * `gap`             -- one-sentence "what they do NOT cover" relative to
//!                          ELAI's production-readiness gates
//!   * `report_finding`  -- the empirical anchor (% / count / source date)
//!   * `archetypes`      -- the subset of [`SaasArchetype`] this entry most
//!                          directly informs (used by the recommender to
//!                          surface relevant prior-art alongside the primary
//!                          archetype recommendation)
//!
//! Constraint per `crates/contracts/tests/e12_production_codegen_contracts.rs`
//! guard 4: NO inline framework version pins in the .rs source. Version
//! literals live exclusively in `workflow/archetypes/*.toml` manifests.
//! These entries cite product+date, not pinned versions.

use super::saas_archetype::SaasArchetype;

/// One immutable row in the prior-art registry. All fields are `'static`
/// string slices because the entire table is a `const`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorArtRef {
    /// Product or report name (e.g. `"v0.dev"`, `"Tenzai CSA Dec 2025"`).
    pub name: &'static str,
    /// Canonical citation URL. Public product landing page OR the
    /// report's source URL.
    pub url: &'static str,
    /// One-sentence "what they do NOT cover" relative to ELAI's
    /// production-readiness gates. Cited downstream by idea-critic
    /// `CriticCategory::Stale` verdicts.
    pub gap: &'static str,
    /// Empirical anchor backing the entry's inclusion. References the
    /// 2024-2026 survey at workflow/improvement-sweep-2026-05-19.
    pub report_finding: &'static str,
    /// Archetypes this entry most directly informs. Iteration surface
    /// for [`entries_for_archetype`].
    pub archetypes: &'static [SaasArchetype],
}

/// The canonical 15-entry registry. 9 product entries + 6 report entries.
///
/// Stability discipline: this table is part of the
/// `e11_b_saas_prior_art_catalog_contract` deterministic check. Adding
/// or removing entries REQUIRES regenerating the contract length
/// expectation. Reordering is fine; the contract is order-agnostic.
pub const PRIOR_ART: &[PriorArtRef] = &[
    // ------------------ Product entries (9) ------------------
    PriorArtRef {
        name: "v0.dev",
        url: "https://v0.app",
        gap: "Next.js scaffold output ships without auth, RLS, or secrets-scan; \
              the produced demo passes the visual review but fails Tenzai's \
              CSRF + headers checklist out of the box.",
        report_finding: "Veracode 2025 GenAI Code Security: 45% of AI-generated \
                         code fails basic security tests.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::ConsumerSocial,
        ],
    },
    PriorArtRef {
        name: "bolt.new",
        url: "https://bolt.new",
        gap: "WebContainer execution surface produces a working demo but ships \
              no first-party RLS / migration-on-prod ban / dependency audit.",
        report_finding: "Tenzai CSA Dec 2025: 100% of 15 surveyed vibe-coded \
                         apps missing CSRF and security headers.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::ConsumerSocial,
        ],
    },
    PriorArtRef {
        name: "lovable",
        url: "https://lovable.dev",
        gap: "End-to-end app emitter; Supabase-tier RLS templates exist but \
              are NOT mandatory in the default flow, and there is no \
              slopsquatting gate on emitted package.json import lines.",
        report_finding: "Help Net Security 2025 slopsquatting paper: 19.7% \
                         package-name hallucination rate across LLMs.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::B2bMarketplace,
        ],
    },
    PriorArtRef {
        name: "replit-agent-3",
        url: "https://replit.com/agent",
        gap: "External API auth flows are a known cliff per Replit's own \
              public retro -- the agent ships demo-grade scaffolds for any \
              third-party OAuth integration.",
        report_finding: "Replit Agent 3 public limitations: external API \
                         auth is the documented Tier-A failure case.",
        archetypes: &[SaasArchetype::SaasDashboard, SaasArchetype::DeveloperApi],
    },
    PriorArtRef {
        name: "cursor-2",
        url: "https://cursor.com",
        gap: "IDE-attached agent; passes SWE-bench Verified at 87.6% but \
              drops to 23% on SWE-Bench Pro (Cemri et al. 2025 cliff).",
        report_finding: "SWE-bench Verified 87.6% vs SWE-Bench Pro 23% \
                         benchmark cliff (arXiv:2509.16941, RAG-indexed).",
        archetypes: &[
            SaasArchetype::DeveloperApi,
            SaasArchetype::InternalTooling,
        ],
    },
    PriorArtRef {
        name: "devin",
        url: "https://devin.ai",
        gap: "Pursues impossible solutions for days under ambiguous greenfield \
              briefs (Answer.AI documented 3/20 success rate, Jan 2025).",
        report_finding: "Answer.AI Devin retrospective Jan 2025: 3/20 \
                         documented success rate on real customer tasks.",
        archetypes: &[
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
        ],
    },
    PriorArtRef {
        name: "codex-cli",
        url: "https://github.com/openai/codex-cli",
        gap: "Local CLI agent; lacks per-role auditor mode and ships no \
              production-readiness checklist.",
        report_finding: "OpenAI codex-cli README: no production-checklist \
                         surface as of 2026-02.",
        archetypes: &[
            SaasArchetype::DeveloperApi,
            SaasArchetype::InternalTooling,
        ],
    },
    PriorArtRef {
        name: "aider",
        url: "https://aider.chat",
        gap: "Edit-format discipline (unified-diff > search/replace > full \
              file) is the strongest empirical lever per Aider's polyglot \
              leaderboard, but Aider itself doesn't gate on auth/RLS/secrets.",
        report_finding: "Aider polyglot leaderboard 2025: edit-format \
                         matters more than model strength.",
        archetypes: &[
            SaasArchetype::DeveloperApi,
            SaasArchetype::InternalTooling,
        ],
    },
    PriorArtRef {
        name: "horizon",
        url: "https://horizon.ai",
        gap: "App-generator with framework-locked templates; no per-archetype \
              taxonomy and no idea-critic equivalent.",
        report_finding: "Horizon public docs 2026-Q1: single-template \
                         framework lock, no archetype-aware emission.",
        archetypes: &[SaasArchetype::SaasDashboard, SaasArchetype::ConsumerSocial],
    },
    // ------------------ Report entries (6) ------------------
    PriorArtRef {
        name: "Tenzai CSA Dec 2025",
        url: "https://cloudsecurityalliance.org/research/tenzai",
        gap: "100% of 15 surveyed vibe-coded apps missing CSRF + security \
              headers -- motivates default-on production-grade auditor.",
        report_finding: "Tenzai CSA Dec 2025 vibe-coding survey: 15/15 apps \
                         shipping without CSRF or security-headers.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::B2bMarketplace,
        ],
    },
    PriorArtRef {
        name: "Veracode 2025 GenAI Code Security",
        url: "https://www.veracode.com/blog/2025-genai-code-security",
        gap: "45% of AI-generated code fails basic security tests across \
              100+ LLMs -- the empirical anchor for the slopsquatting + \
              OWASP-top-10 gates.",
        report_finding: "Veracode 2025 GenAI Code Security: 45% fail-rate \
                         on basic security tests across 100+ LLMs.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::B2bMarketplace,
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
            SaasArchetype::ConsumerSocial,
        ],
    },
    PriorArtRef {
        name: "Escape.tech / Baytech 2025",
        url: "https://escape.tech/blog/baytech-2025",
        gap: "1,400 apps scanned, 65% with security issues, 400+ exposed \
              secrets -- motivates the secrets-scan + trufflehog gate.",
        report_finding: "Escape.tech / Baytech 2025: 1,400 apps, 65% with \
                         security issues, 400+ secrets exposed.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::B2bMarketplace,
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
            SaasArchetype::ConsumerSocial,
        ],
    },
    PriorArtRef {
        name: "Help Net Security 2025 slopsquatting",
        url: "https://www.helpnetsecurity.com/2025/slopsquatting",
        gap: "19.7% package-name hallucination rate across LLMs -- motivates \
              the per-emit package-existence verifier (sibling slice).",
        report_finding: "Help Net Security 2025: 19.7% slopsquatting rate \
                         on emitted import lines across surveyed LLMs.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::B2bMarketplace,
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
            SaasArchetype::ConsumerSocial,
        ],
    },
    PriorArtRef {
        name: "Answer.AI Devin retrospective Jan 2025",
        url: "https://www.answer.ai/posts/2025-01-devin.html",
        gap: "Documented 3/20 success rate -- motivates the anti-spiral \
              hard-abort wall-clock budget on ambiguous greenfield work.",
        report_finding: "Answer.AI Jan 2025: 3/20 Devin success rate \
                         (days-long impossible-task chase on ambiguous briefs).",
        archetypes: &[
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
        ],
    },
    PriorArtRef {
        name: "Cognition Devin 2025 review",
        url: "https://www.cognition.ai/blog/devin-2025-review",
        gap: "\"Junior at execution, can't tackle ambiguity end-to-end\" -- \
              motivates the scope_ceiling table + 4 KNOWN_DEMO_VS_DEPLOY \
              triggers (sibling slice).",
        report_finding: "Cognition 2025 review of Devin: ambiguity-handling \
                         is the documented Tier-A failure mode.",
        archetypes: &[
            SaasArchetype::SaasDashboard,
            SaasArchetype::InternalTooling,
            SaasArchetype::DeveloperApi,
        ],
    },
];

/// Read-only registry surface. Currently a unit type — the table is
/// global `const` so all lookups are O(N) scans over 15 rows (cache-line
/// fits the whole slice; no need for hashing).
#[derive(Debug, Default, Clone, Copy)]
pub struct PriorArtRegistry;

impl PriorArtRegistry {
    /// Returns the prior-art entries tagged with the given archetype.
    /// Order matches [`PRIOR_ART`] declaration order (stable for tests).
    #[must_use]
    pub fn entries_for_archetype(archetype: SaasArchetype) -> Vec<&'static PriorArtRef> {
        PRIOR_ART
            .iter()
            .filter(|e| e.archetypes.contains(&archetype))
            .collect()
    }

    /// Total count of registered prior-art entries.
    #[must_use]
    pub fn len() -> usize {
        PRIOR_ART.len()
    }

    /// Iterate over the full registry in declaration order.
    pub fn iter() -> impl Iterator<Item = &'static PriorArtRef> {
        PRIOR_ART.iter()
    }
}
