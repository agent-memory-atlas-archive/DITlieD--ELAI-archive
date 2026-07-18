//! E-37 U2 — DefaultLlmInjectionClassifier (PromptArmor tier-2).
//! RECON[sidecar]: GPU-PAUSED-BY-OWNER — no :8181 probe; fail-open to tier-1.

use elai_safety::prompt_armor::{
    ClassifierVerdict, HeuristicClassifier, InjectionClassifier, PromptArmorError,
};
use elai_safety::spotlight::SpotlightedSpan;
use std::future::Future;
use std::pin::Pin;

/// LLM-escalation classifier. Under GPU-PAUSED-BY-OWNER this always
/// fail-opens to [`HeuristicClassifier`] + warn! (no local sidecar probe).
pub(crate) struct DefaultLlmInjectionClassifier {
    heuristic: HeuristicClassifier,
}

impl DefaultLlmInjectionClassifier {
    pub(crate) fn new() -> Self {
        Self {
            heuristic: HeuristicClassifier,
        }
    }
}

impl InjectionClassifier for DefaultLlmInjectionClassifier {
    fn classify<'a>(
        &'a self,
        span: &'a SpotlightedSpan,
    ) -> Pin<Box<dyn Future<Output = Result<ClassifierVerdict, PromptArmorError>> + Send + 'a>> {
        Box::pin(async move {
            // RECON[sidecar]: GPU-PAUSED-BY-OWNER — no :8181 probe; fail-open to tier-1
            tracing::warn!(
                target: "llm_injection_classifier",
                attribution = "heuristic-fallback",
                "GPU-PAUSED-BY-OWNER; fail-open to HeuristicClassifier (no :8181)"
            );
            self.heuristic.classify(span).await
        })
    }
}

/// Tier-1 regex then escalate Suspect to DefaultLlmInjectionClassifier.
pub(crate) async fn classify_with_llm_escalation(text: &str) -> ClassifierVerdict {
    use elai_safety::prompt_armor::classify_with_regex;
    let tier1 = classify_with_regex(text);
    match &tier1 {
        ClassifierVerdict::Suspect { .. } => {
            // Build a minimal spotlighted span via wrap_spotlight for the trait API.
            use elai_core::{EntropySource, SeededRng};
            use elai_safety::{wrap_spotlight, Provenance};
            let provenance = Provenance::from_web("https://e37.local/llm-escalation");
            let mut entropy = SeededRng::new(0xE37u64);
            match wrap_spotlight(text, &provenance, &mut entropy) {
                Ok(span) => {
                    let classifier = DefaultLlmInjectionClassifier::new();
                    classifier
                        .classify(&span)
                        .await
                        .unwrap_or(tier1)
                }
                Err(_) => tier1,
            }
        }
        _ => tier1,
    }
}
