//! J-42 — T1 distill-reset contract tests.
//!
//! These tests exercise the `ChatReflectPipeline::run` refine loop seam in
//! `.elai_cc/crates/reasoning/src/chat_reflect.rs`, reached in production by
//! `.elai_cc/crates/cli/src/live_cli.rs:1821` and the
//! `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:7206` region.
//!
//! Falsification mutation: deleting the trigger guard in the loop body must
//! turn test 2 RED. The default test guarantees shipped `/reason` behavior is
//! unchanged until the EXP-3 measurement flips the trigger on.

use std::sync::Arc;

use elai_reasoning::chat_reflect::{
    ChatFuture, ChatInferenceClient, ChatReflectConfig, ChatReflectPipeline,
};

struct CapturingClient {
    responses: std::sync::Mutex<Vec<String>>,
    sent: std::sync::Mutex<Vec<String>>,
}

impl CapturingClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
            sent: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn sent(&self) -> Vec<String> {
        self.sent.lock().expect("sent lock poisoned").clone()
    }
}

impl ChatInferenceClient for CapturingClient {
    fn complete<'a>(&'a self, prompt: &'a str, _max_tokens: u32) -> ChatFuture<'a, String> {
        Box::pin(async move {
            self.sent
                .lock()
                .expect("sent lock poisoned")
                .push(prompt.to_string());
            let mut responses = self.responses.lock().expect("responses lock poisoned");
            if responses.is_empty() {
                return Err(elai_reasoning::chat_reflect::ReflectError::Inference(
                    "mock exhausted".into(),
                ));
            }
            Ok(responses.remove(0))
        })
    }
}

struct ArcClient(Arc<CapturingClient>);

impl ChatInferenceClient for ArcClient {
    fn complete<'a>(&'a self, prompt: &'a str, max_tokens: u32) -> ChatFuture<'a, String> {
        self.0.complete(prompt, max_tokens)
    }
}

fn responses(auditor: &str, distillates: &[&str]) -> Vec<String> {
    let mut out = vec![r#"{"class":"problem"}"#.into()];
    out.extend((0..4).map(|i| format!("candidate {i}")));
    out.push(auditor.into());
    out.extend(distillates.iter().map(|s| (*s).into()));
    out.push(auditor.into());
    out
}

fn is_distill(prompt: &str) -> bool {
    prompt.contains("Compress the audit reasoning")
}

#[tokio::test]
async fn default_configuration_never_distills() {
    // Load-bearing guarantee: shipped `/reason` behavior is unchanged until
    // the EXP-3 measurement flips the trigger on.
    let client = Arc::new(CapturingClient::new(responses(
        "0|candidate zero is best",
        &[],
    )));
    let pipeline = ChatReflectPipeline::new(ChatReflectConfig::default(), None);
    let arc_client = ArcClient(client.clone());
    let output = pipeline.run("j42-default".into(), "A question.", &arc_client).await;
    let sent = client.sent();

    assert!(!output.skipped);
    assert_eq!(sent.iter().filter(|p| is_distill(p)).count(), 0);
    assert!(sent.last().expect("refine request exists").contains("Previous verdict:"));
    assert_eq!(sent.len(), 7);
}

#[tokio::test]
async fn trigger_fires_and_reseed_pins_original_question() {
    let mut config = ChatReflectConfig::default();
    config.budget_cap_tokens = 8;
    config.distill_reset_trigger_pct = 75;
    let question = "Explain why this lock is poisoned.";
    let client = Arc::new(CapturingClient::new(responses(
        "0|candidate zero is best overall",
        &["Trace compressed to essentials."],
    )));
    let pipeline = ChatReflectPipeline::new(config, None);
    let arc_client = ArcClient(client.clone());
    let output = pipeline.run("j42-trigger".into(), question, &arc_client).await;
    let sent = client.sent();
    let last = sent.last().expect("reseed request exists");

    assert!(!output.skipped);
    assert_eq!(sent.iter().filter(|p| is_distill(p)).count(), 1);
    assert!(last.contains("Trace compressed to essentials."));
    assert!(last.contains(question));
    assert!(!last.contains("Previous verdict:"));
}

#[tokio::test]
async fn hedge_stripped_distillate_is_reasked_once_then_abandoned() {
    let mut config = ChatReflectConfig::default();
    config.budget_cap_tokens = 8;
    config.distill_reset_trigger_pct = 75;
    let client = Arc::new(CapturingClient::new(responses(
        "0|maybe candidate zero, unverified",
        &["Candidate zero is correct.", "Candidate zero is correct after review."],
    )));
    let pipeline = ChatReflectPipeline::new(config, None);
    let arc_client = ArcClient(client.clone());
    let output = pipeline.run("j42-hedge".into(), "A question.", &arc_client).await;
    let sent = client.sent();
    let last = sent.last().expect("fallback refine request exists");

    assert!(!output.skipped);
    assert_eq!(sent.iter().filter(|p| is_distill(p)).count(), 2);
    assert!(last.contains("Previous verdict:"));
    assert!(!last.contains("Candidate zero is correct."));
    assert!(!last.contains("Candidate zero is correct after review."));
}
