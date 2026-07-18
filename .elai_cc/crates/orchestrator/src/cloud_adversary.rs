//! P0-1 — CloudAdversary: production Anthropic-backed [`crate::adversary::Adversary`] impl.
//!
//! Extracted from `adversary.rs` (MR-1 file-size fix). Re-exported from
//! `crate::adversary` so all existing callers remain valid.
//!
//! See the `adversary` module doc for the D-13 cross-family verification
//! grounding and the full trait contract.

use crate::adversary::{Adversary, AdversaryConfig, AdversaryError, CounterEvidenceSchema};
use crate::claim::Claim;
use elai_providers::{InputMessage, MessageRequest, OutputContentBlock};

// ---------------------------------------------------------------------------
// CloudAdversary — production Anthropic-backed Adversary impl (P0-1)
// ---------------------------------------------------------------------------

/// P0-1 — production [`Adversary`] that calls the Anthropic API with the
/// [`elai_providers::AnthropicClient`] provided at construction time. Emits from
/// [`crate::adversary::ModelFamily::Anthropic`], so it satisfies the cross-family
/// constraint when the worker runs on `ModelFamily::Local` (Sidecar/Qwen).
///
/// On any parse or network error the method returns `Ok(schema)` with
/// `severity == 0.0` and empty `counter_evidence` (contract: on
/// no-counter-found return robust schema, not an error). The caller's
/// two-gate logic (`resolved.adversary_runtime.is_some()` AND
/// `ELAI_ADVERSARY_ENABLE=1`) still runs; a soft failure just means the
/// claim is treated as robust for this round.
///
/// Constructed by `build_task_runner_wiring` when
/// `ELAI_ALLOW_CLOUD_ADVERSARY=1` is set AND
/// `route_cross_family` returns `Ok`.
pub struct CloudAdversary {
    client: elai_providers::AnthropicClient,
    model: String,
}

impl CloudAdversary {
    /// Build a `CloudAdversary` from an existing [`elai_providers::AnthropicClient`]
    /// and a model identifier (e.g. `"claude-sonnet-4-6"`).
    #[must_use]
    pub fn new(client: elai_providers::AnthropicClient, model: impl Into<String>) -> Self {
        Self { client, model: model.into() }
    }

    /// Convenience constructor that calls [`elai_providers::AnthropicClient::from_env`].
    /// Returns `Err` when `ANTHROPIC_API_KEY` is absent or malformed.
    pub fn from_env(model: impl Into<String>) -> Result<Self, elai_providers::ApiError> {
        Ok(Self::new(elai_providers::AnthropicClient::from_env()?, model))
    }

    /// Robust (no-counter) fallback schema — used when the network call
    /// fails or the response cannot be parsed.
    fn robust_schema(claim: &Claim) -> CounterEvidenceSchema {
        CounterEvidenceSchema {
            counter_id: "cloud-adversary-fallback".into(),
            original_claim_id: claim.claim_id.clone(),
            counter_text: String::new(),
            counter_evidence: Vec::new(),
            severity: 0.0,
        }
    }
}

impl Adversary for CloudAdversary {
    async fn generate_counter(
        &self,
        claim: &Claim,
        _config: &AdversaryConfig,
    ) -> Result<CounterEvidenceSchema, AdversaryError> {
        // Prompt: ask the cloud model for a structured JSON counter-evidence
        // block.  The JSON schema mirrors CounterEvidenceSchema so the
        // downstream parse step works without a separate mapping.
        let prompt = format!(
            "You are a rigorous adversarial reviewer. Given the claim below, produce \
             a JSON object with the following fields:\n\
             - counter_id (string): a short kebab-case id for this counter-claim\n\
             - original_claim_id (string): copy of the claim id below\n\
             - counter_text (string): the strongest plausible counter-claim\n\
             - counter_evidence (array of strings): evidence references\n\
             - severity (number 0.0-1.0): how strongly this counter refutes the claim \
               (0.0 = claim is robust, 1.0 = claim is fully refuted)\n\n\
             If no credible counter exists, return severity=0.0 and empty arrays.\n\
             Output ONLY the JSON object, no prose.\n\n\
             Claim id: {}\nClaim text: {}",
            claim.claim_id, claim.text
        );

        let request = MessageRequest {
            model: self.model.clone(),
            max_tokens: 512,
            messages: vec![InputMessage::user_text(prompt)],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
            // I-26: effort is not set for adversarial cloud calls.
            effort: None,
            chat_template_kwargs: None,
        };

        let response = match self.client.send_message(&request).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    claim_id = %claim.claim_id,
                    "CloudAdversary: send_message failed; treating claim as robust"
                );
                return Ok(Self::robust_schema(claim));
            }
        };

        // Extract the first text block from the response.
        let text = response
            .content
            .iter()
            .find_map(|b| {
                if let OutputContentBlock::Text { text } = b {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        // Parse the JSON into a CounterEvidenceSchema.  On any parse failure
        // treat the claim as robust (per Adversary trait contract §2).
        match serde_json::from_str::<CounterEvidenceSchema>(text) {
            Ok(schema) => Ok(schema),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    claim_id = %claim.claim_id,
                    "CloudAdversary: JSON parse failed; treating claim as robust"
                );
                Ok(Self::robust_schema(claim))
            }
        }
    }
}
