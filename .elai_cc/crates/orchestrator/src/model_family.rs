//! D-13 — ModelFamily: closed enum of verifier model families.
//!
//! Extracted from `adversary.rs` (MR-1 file-size fix, SP-3A). Re-exported
//! from `crate::adversary` so all existing callers remain valid.
//!
//! See the `adversary` module doc for the D-13 cross-family verification
//! grounding and the full trait contract.

use serde::{Deserialize, Serialize};
use elai_providers::ProviderKind;

// ---------------------------------------------------------------------------
// ModelFamily — closed enum of verifier model families
// ---------------------------------------------------------------------------

/// Closed enum of model families used by the heterogeneous-verifier
/// check. `Other(String)` is intentionally provided as an escape hatch
/// for future families not yet first-class in ELAI's deployment — same
/// shape as `crate::adversary::ModelFamily` in the D-13 plan, but the
/// closed variants are the only ones used by
/// [`heterogeneous_verifier_check`]'s built-in cross-family logic.
///
/// Two families are "different" iff their snake-case wire labels differ.
/// `Other(s1)` vs `Other(s2)` is "different" iff `s1 != s2`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    /// Anthropic Claude family (Sonnet / Opus / Haiku across versions).
    Anthropic,
    /// OpenAI GPT family (GPT-4 / 4o / 4.5 / 5 across versions).
    OpenAi,
    /// Meta Llama family (Llama 2 / 3 / 4 across sizes).
    Meta,
    /// Mistral / Mixtral family.
    Mistral,
    /// DeepSeek family (V2 / V3 / Coder across sizes).
    DeepSeek,
    /// Local on-device model (Qwen2.5-Coder-7B in ELAI's deployment).
    /// Treated as a single family regardless of the specific local
    /// checkpoint — local-vs-local pairings are same-family by design.
    Local,
    /// Escape hatch for families not yet first-class. Pairs of
    /// `Other(a)` and `Other(b)` are heterogeneous iff `a != b`.
    Other(String),
}

impl ModelFamily {
    /// Stable wire label used in SSB rows + dashboard rendering.
    /// `Other(s)` borrows `s`; other variants return a 'static literal.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::Meta => "meta",
            Self::Mistral => "mistral",
            Self::DeepSeek => "deepseek",
            Self::Local => "local",
            Self::Other(s) => s.as_str(),
        }
    }

    /// D-13 closure GAP-2 — bridge a [`elai_providers::ProviderKind`]
    /// classification into the adversary's [`ModelFamily`] partition.
    ///
    /// This replaces caller-asserted family with a runtime classification
    /// that defers to the shared provider detection layer
    /// ([`elai_providers::detect_provider_kind`]). The two enums are
    /// intentionally disjoint surfaces (`ProviderKind` is the wire-level
    /// transport; `ModelFamily` is the heterogeneous-verifier semantic
    /// partition per arxiv:2504.16760 T1), but the mapping is total:
    /// every `ProviderKind` lands on exactly one `ModelFamily`. The
    /// `Ollama` and `Vllm` providers collapse to `ModelFamily::Local`
    /// because both run on-device — same-family-as-worker for the
    /// default Qwen2.5-Coder-7B local deployment.
    ///
    /// Per D-13 plan task 2 cross-family check: callers MUST NOT use the
    /// caller-asserted family when the runtime can be classified. The
    /// bridge is preferred so a misconfigured cloud Sonnet caller cannot
    /// accidentally pair against another Anthropic adversary.
    #[must_use]
    pub fn from_provider_kind(kind: ProviderKind) -> Self {
        match kind {
            ProviderKind::Anthropic => Self::Anthropic,
            ProviderKind::OpenAi => Self::OpenAi,
            // Xai uses the OpenAI-compat transport but is a distinct
            // model family. The closed enum does not have an Xai variant
            // so it lands on `Other("xai")` for the cross-family check.
            ProviderKind::Xai => Self::Other("xai".into()),
            // Ollama + Vllm both host local-weights models. Treat both
            // as `Local` per the existing adversary.rs:75-78
            // doc-comment ("local-vs-local pairings are same-family by
            // design"). This is intentional: pairing two on-device
            // models — regardless of checkpoint — is same-family for
            // the cross-family guard.
            ProviderKind::Ollama => Self::Local,
            #[cfg(feature = "vllm")]
            ProviderKind::Vllm => Self::Local,
            // SP-3A: bmapi's gpt-5.5 proxy. codex-vm's own
            // ~/.codex/config.toml declares `model_provider = "OpenAI"` for
            // this exact model, and the live catalog it serves (gpt-5.4,
            // gpt-5.5, gpt-image-2, codex-auto-review; verified 2026-07-05)
            // is entirely OpenAI-branded naming reached over a different
            // transport/credential (a personal proxy, not api.openai.com
            // directly) - same real-world foundation-model family as
            // `ProviderKind::OpenAi`, so it maps the same way for the
            // cross-family check's purpose (avoid a same-family blind spot,
            // not a same-transport one).
            ProviderKind::Gpt55Proxy => Self::OpenAi,
        }
    }

    /// D-13 closure GAP-2 — model_id substring classifier used when
    /// only the model identifier string is available (e.g. local
    /// sidecar boot where no `ProviderKind` is yet computed). Mirrors
    /// the `_PPM_MODEL_HINTS` substring approach from
    /// [`crate::ppm_client`] (D-2 precedent).
    ///
    /// Substring matching is case-insensitive and runs in declaration
    /// order; first match wins. Falls back to `Local` (rather than
    /// `Other`) because every shipped local model_id today is a Qwen
    /// or Llama checkpoint — both correctly classified as `Local` per
    /// the same-family-on-device rationale above.
    ///
    /// Use this when only `model_id` is in hand. Prefer
    /// [`Self::from_provider_kind`] when a `ProviderKind` is already
    /// computed (cheaper + already-classified).
    #[must_use]
    pub fn from_model_id(model_id: &str) -> Self {
        let lower = model_id.trim().to_ascii_lowercase();
        if lower.is_empty() {
            return Self::Local;
        }
        // Cloud-provider id hints first (most specific). The substring
        // set mirrors D-2's PpmAdapter model hints + the provider
        // crate's MODEL_REGISTRY canonical aliases.
        if lower.contains("claude") || lower.starts_with("anthropic") {
            return Self::Anthropic;
        }
        if lower.contains("gpt") || lower.starts_with("openai") {
            return Self::OpenAi;
        }
        if lower.contains("grok") || lower.starts_with("xai") {
            return Self::Other("xai".into());
        }
        if lower.contains("deepseek") {
            return Self::DeepSeek;
        }
        if lower.contains("llama") {
            // Llama-family hosted locally OR via Meta endpoints. The
            // closed-enum `Meta` variant covers both — heterogeneous
            // check treats Meta-Llama as distinct from Local-Qwen.
            return Self::Meta;
        }
        if lower.contains("mistral") || lower.contains("mixtral") {
            return Self::Mistral;
        }
        if lower.contains("gemma") {
            // Gemma is Google's open-weights family; no closed-enum
            // slot yet, so it lands on `Other("gemma")` so a
            // future-deployment Gemma adversary still satisfies the
            // cross-family check against Local-Qwen.
            return Self::Other("gemma".into());
        }
        if lower.contains("qwen") {
            return Self::Local;
        }
        // Unknown id — default to Local to be conservative. A Local
        // adversary against a Local worker triggers SameFamilyPairing
        // in `run_adversarial_pass` so the misclassification is
        // observable rather than silent.
        Self::Local
    }
}
